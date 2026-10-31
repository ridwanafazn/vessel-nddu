// nmea.c (modified to support variable-length sentence IDs like PANZHRP and digits like LR1)
// Keeps standard NMEA0183 compatible.

#include "nmea.h"
#include "parser.h"
#include "parser_types.h"

#include <ctype.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define ARRAY_LENGTH(a) (sizeof a / sizeof (a[0]))

/* Standard NMEA sentence ID: 5 chars (TTSSS). Allow longer custom IDs too. */
#ifndef NMEA_TYPE_MIN_LEN
#define NMEA_TYPE_MIN_LEN 5
#endif

#ifndef NMEA_TYPE_MAX_LEN
#define NMEA_TYPE_MAX_LEN 16 /* adjust if you need longer than PANZHRP */
#endif

static int
_is_nmea_id_char(char c)
{
	/* Allow uppercase letters and digits (e.g., LR1) */
	return ((c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9'));
}

static int
_is_hex2(const char *p)
{
	if (!p) return 0;
	return isxdigit((unsigned char)p[0]) && isxdigit((unsigned char)p[1]);
}

/**
 * Check if a value is not NULL and not empty.
 *
 * Returns 0 if set, otherwise -1.
 */
static int
_is_value_set(const char *value)
{
	if (NULL == value || '\0' == *value) {
		return -1;
	}

	return 0;
}

/* Find pointer to '\r' (assumes validate() enforces CRLF). */
static const char *
_find_cr(const char *sentence, size_t length)
{
	if (!sentence || length < 2) return NULL;
	return sentence + (length - 2); /* should be '\r' */
}

/**
 * Parse the ID span between '$' and the first comma.
 * Accepts variable length (>=5) and allows digits.
 */
static int
_get_type_span(const char *sentence, size_t length,
               const char **type_start, size_t *type_len,
               const char **comma_pos)
{
	if (!sentence || length < 3) return -1;
	if (sentence[0] != '$') return -1;

	const char *cr = _find_cr(sentence, length);
	if (!cr) return -1;

	const char *comma = strchr(sentence, ',');
	if (!comma) return -1;
	if (comma >= cr) return -1;

	const char *start = sentence + 1;
	size_t len = (size_t)(comma - start);

	if (len < NMEA_TYPE_MIN_LEN || len > NMEA_TYPE_MAX_LEN) return -1;

	for (size_t i = 0; i < len; i++) {
		char c = start[i];
		if (!_is_nmea_id_char(c)) {
			return -1;
		}
	}

	if (type_start) *type_start = start;
	if (type_len) *type_len = len;
	if (comma_pos) *comma_pos = comma;
	return 0;
}

/**
 * Crop a sentence from the type word and checksum.
 *
 * Removes everything up to and including the first comma.
 * Removes checksum part if present ("*hh").
 * Removes CRLF from the end.
 *
 * sentence is a validated NMEA sentence string.
 * length is the char length of the sentence string.
 *
 * Returns pointer (char *) to the payload string (first field after comma).
 *
 * NOTE: modifies sentence buffer in-place.
 */
static char *
_crop_sentence(char *sentence, size_t length)
{
	if (!sentence || length < 2) return NULL;

	const char *type_start = NULL;
	size_t type_len = 0;
	const char *comma_c = NULL;

	if (_get_type_span(sentence, length, &type_start, &type_len, &comma_c) != 0) {
		return NULL;
	}

	char *comma = (char *)comma_c;
	char *payload = comma + 1;

	/* Null terminate at '\r' (strip CRLF) */
	char *cr = sentence + (length - 2);
	*cr = '\0';

	/* Remove checksum, if there is one */
	char *star = strchr(payload, '*');
	if (star) {
		*star = '\0';
	}

	return payload;
}

/**
 * Splits a string by comma.
 *
 * string is the string to split, will be manipulated. Needs to be
 *        null-terminated.
 * values is a char pointer array that will be filled with pointers to the
 *        splitted values in the string.
 * max_values is the maximum number of values to be parsed.
 *
 * Returns the number of values found in string.
 */
static int
_split_string_by_comma(char *string, char **values, int max_values)
{
	int i = 0;

	values[i++] = string;
	while (i < max_values && NULL != (string = strchr(string, ','))) {
		*string = '\0';
		values[i++] = ++string;
	}

	return i;
}

/**
 * Initiate the NMEA library and load the parser modules.
 *
 * This function will be called before the main() function.
 */
void __attribute__ ((constructor)) nmea_init(void);
void nmea_init()
{
	nmea_load_parsers();
}

/**
 * Unload the parser modules.
 *
 * This function will be called after the exit() function.
 */
void __attribute__ ((destructor)) nmea_cleanup(void);
void nmea_cleanup()
{
	nmea_unload_parsers();
}

nmea_t
nmea_get_type(const char *sentence)
{
	if (NULL == sentence) {
		return NMEA_UNKNOWN;
	}
	nmea_parser_module_s *parser = nmea_get_parser_by_sentence(sentence);
	if (NULL == parser) {
		return NMEA_UNKNOWN;
	}

	return parser->parser.type;
}

uint8_t
nmea_get_checksum(const char *sentence)
{
	const char *n = sentence + 1;
	uint8_t chk = 0;

	/* While current char isn't '*' or sentence ending (newline) */
	while ('*' != *n && NMEA_END_CHAR_1 != *n && '\0' != *n) {
		chk ^= (uint8_t) *n;
		n++;
	}

	return chk;
}

/* Position-independent checksum detection (instead of length-5). */
int
nmea_has_checksum(const char *sentence, size_t length)
{
	if (!sentence || length < 7) return -1;

	const char *cr = _find_cr(sentence, length);
	if (!cr) return -1;

	const char *star = strchr(sentence, '*');
	if (!star) return -1;
	if (star >= cr) return -1;

	/* Need exactly two hex digits after '*' before '\r' */
	if ((star + 2) >= cr) return -1;
	if (!_is_hex2(star + 1)) return -1;

	return 0;
}

int
nmea_validate(const char *sentence, size_t length, int check_checksum)
{
	/* should have atleast 9 characters */
	if (9 > length) {
		return -1;
	}

	/* should be less or equal to 82 characters */
	if (NMEA_MAX_LENGTH < length) {
		return -1;
	}

	/* should start with $ */
	if ('$' != *sentence) {
		return -1;
	}

	/* should end with \r\n, or other... */
	if (NMEA_END_CHAR_2 != sentence[length - 1] || NMEA_END_CHAR_1 != sentence[length - 2]) {
		return -1;
	}

	/* should have an ID between '$' and first comma (>=5 chars), allow digits */
	const char *type_start = NULL;
	size_t type_len = 0;
	const char *comma = NULL;
	if (_get_type_span(sentence, length, &type_start, &type_len, &comma) != 0) {
		return -1;
	}

	/* check for checksum */
	if (1 == check_checksum && 0 == nmea_has_checksum(sentence, length)) {
		uint8_t actual_chk;
		uint8_t expected_chk;
		char checksum[3];

		const char *star = strchr(sentence, '*');
		if (!star) return -1;

		checksum[0] = star[1];
		checksum[1] = star[2];
		checksum[2] = '\0';

		actual_chk = nmea_get_checksum(sentence);
		expected_chk = (uint8_t) strtol(checksum, NULL, 16);
		if (expected_chk != actual_chk) {
			return -1;
		}
	}

	return 0;
}

void
nmea_free(nmea_s *data)
{
	nmea_parser_module_s *parser;

	if (NULL == data) {
		return;
	}

	parser = nmea_get_parser_by_type(data->type);
	if (NULL == parser) {
		return;
	}

	parser->free_data(data);
}

nmea_s *
nmea_parse(char *sentence, size_t length, int check_checksum)
{
	unsigned int n_vals, val_index;
	char *value, *val_string;
	char *values[255];
	nmea_parser_module_s *parser;
	nmea_t type;

	/* Validate sentence string */
	if (-1 == nmea_validate(sentence, length, check_checksum)) {
		return (nmea_s *) NULL;
	}

	type = nmea_get_type(sentence);
	if (NMEA_UNKNOWN == type) {
		return (nmea_s *) NULL;
	}

	/* Crop sentence from type word and checksum */
	val_string = _crop_sentence(sentence, length);
	if (NULL == val_string) {
		return (nmea_s *) NULL;
	}

	/* Split the sentence into values */
	n_vals = _split_string_by_comma(val_string, values, ARRAY_LENGTH(values));
	if (0 == n_vals) {
		return (nmea_s *) NULL;
	}

	/* Get the right parser */
	parser = nmea_get_parser_by_type(type);
	if (NULL == parser) {
		return (nmea_s *) NULL;
	}

	/* Allocate memory for parsed data */
	parser->allocate_data((nmea_parser_s *) parser);
	if (NULL == parser->parser.data) {
		return (nmea_s *) NULL;
	}

	/* Set default values */
	parser->set_default((nmea_parser_s *) parser);
	parser->errors = 0;

	/* Loop through the values and parse them... */
	for (val_index = 0; val_index < n_vals; val_index++) {
		value = values[val_index];
		if (-1 == _is_value_set(value)) {
			continue;
		}

		if (-1 == parser->parse((nmea_parser_s *) parser, value, val_index)) {
			parser->errors++;
		}
	}

	parser->parser.data->type = type;
	parser->parser.data->errors = parser->errors;

	return parser->parser.data;
}
