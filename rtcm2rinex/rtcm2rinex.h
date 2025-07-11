/*------------------------------------------------------------------------------
* rtcm2rinex.h : RTCM to RINEX converter API header
*
* This is a simplified API for RTCM to RINEX conversion based on RTKLIB
*-----------------------------------------------------------------------------*/
#ifndef RTCM2RINEX_H
#define RTCM2RINEX_H

#include "rtklib.h"

#ifdef __cplusplus
extern "C" {
#endif

/* Initialize conversion context ------------------------------------------------
* initialize RTCM to RINEX conversion context
* args   : rtcm_t *rtcm      IO  RTCM control struct
* return : status (1:ok 0:error)
* notes  : this function must be called before starting data conversion
*-----------------------------------------------------------------------------*/
int rtcm2rinex_init(rtcm_t *rtcm);

/* Process RTCM data -----------------------------------------------------------
* process RTCM data
* args   : rtcm_t *rtcm      IO  RTCM control struct
*          uint8_t data      I   RTCM data (1 byte)
* return : status (-1:error, 0:no message, 1:observation data, 2:nav data,
*                  3:station info, 9:debug)
* notes  : call this function repeatedly to process RTCM data stream
*-----------------------------------------------------------------------------*/
int rtcm2rinex_procdata(rtcm_t *rtcm, uint8_t data);

/* Process RTCM file -----------------------------------------------------------
* process RTCM file
* args   : rtcm_t *rtcm      IO  RTCM control struct
*          const char *file  I   RTCM file path
* return : status (1:ok 0:error)
* notes  : this function reads an entire RTCM file and processes it
*-----------------------------------------------------------------------------*/
int rtcm2rinex_procfile(rtcm_t *rtcm, const char *file);

/* Convert to RINEX -----------------------------------------------------------
* convert RTCM data to RINEX format
* args   : rtcm_t *rtcm      IO  RTCM control struct
*          rnxopt_t *opt     I   RINEX options
*          const char *file  I   output RINEX file path
* return : status (1:ok 0:error)
*-----------------------------------------------------------------------------*/
int rtcm2rinex_convert(rtcm_t *rtcm, rnxopt_t *opt, const char *outfile);

/* Simple conversion from RTCM file to RINEX file ------------------------------
* convert RTCM file to RINEX file with default options
* args   : const char *rtcmfile  I   input RTCM file path
*          const char *outfile   I   output RINEX file path
*          double ver            I   RINEX version (2.10,2.11,2.12,3.00,3.01,3.02,3.03,3.04)
* return : status (1:ok 0:error)
*-----------------------------------------------------------------------------*/
int rtcm2rinex_simple(const char *rtcmfile, const char *outfile, double ver);

#ifdef __cplusplus
}
#endif

#endif /* RTCM2RINEX_H */ 