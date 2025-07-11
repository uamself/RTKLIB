/*------------------------------------------------------------------------------
* example.c : RTCM to RINEX converter API usage example
*
* This example shows how to use the RTCM to RINEX API for different scenarios
*-----------------------------------------------------------------------------*/
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "rtcm2rinex.h"

/* Example 1: Simple conversion from file to file -------------------------------*/
static void example1(const char *rtcmfile, const char *rinexfile)
{
    printf("Example 1: Simple file to file conversion\n");
    
    /* convert RTCM file to RINEX with default settings (RINEX 3.04) */
    if (!rtcm2rinex_simple(rtcmfile, rinexfile, 3.04)) {
        printf("Conversion failed\n");
        return;
    }
    
    printf("Converted %s to %s successfully\n", rtcmfile, rinexfile);
}

/* Example 2: Process RTCM data stream -----------------------------------------*/
static void example2(const char *rtcmfile, const char *rinexfile)
{
    FILE *fp;
    rtcm_t rtcm={0};
    uint8_t data;
    int ret;
    
    printf("Example 2: Process RTCM data stream\n");
    
    /* initialize RTCM decoder */
    if (!rtcm2rinex_init(&rtcm)) {
        printf("RTCM initialization failed\n");
        return;
    }
    
    /* open RTCM file */
    if (!(fp=fopen(rtcmfile,"rb"))) {
        printf("Cannot open %s\n", rtcmfile);
        return;
    }
    
    /* process RTCM data stream byte by byte */
    while (fread(&data,1,1,fp) == 1) {
        ret = rtcm2rinex_procdata(&rtcm, data);
        
        /* output processing status for observation data */
        if (ret == 1) {
            printf("Processed observation data: %s sat=%d\n", 
                   time_str(rtcm.time, 0), rtcm.obs.n);
        }
        /* output processing status for navigation data */
        else if (ret == 2) {
            printf("Processed navigation data: %s\n", time_str(rtcm.time, 0));
        }
    }
    
    /* close RTCM file */
    fclose(fp);
    
    /* prepare RINEX options */
    rnxopt_t opt={{0}};
    opt.rnxver=304; /* RINEX 3.04 */
    opt.obstype=OBSTYPE_PR|OBSTYPE_CP; /* pseudorange and carrier phase */
    opt.navsys=SYS_GPS|SYS_GLO|SYS_GAL|SYS_QZS|SYS_SBS|SYS_CMP|SYS_IRN; /* all systems */
    sprintf(opt.prog,"%s","RTCM2RINEX");
    sprintf(opt.runby,"%s","EXAMPLE");
    
    /* convert to RINEX */
    if (!rtcm2rinex_convert(&rtcm, &opt, rinexfile)) {
        printf("RINEX conversion failed\n");
        free_rtcm(&rtcm);
        return;
    }
    
    printf("Converted to %s successfully\n", rinexfile);
    
    /* free resources */
    free_rtcm(&rtcm);
}

/* Example 3: Customized conversion with specific options ---------------------*/
static void example3(const char *rtcmfile, const char *rinexfile)
{
    rnxopt_t opt={{0}};
    rtcm_t rtcm={0};
    char *ofile[9]={0};
    
    printf("Example 3: Customized conversion\n");
    
    /* initialize RTCM decoder */
    if (!rtcm2rinex_init(&rtcm)) {
        printf("RTCM initialization failed\n");
        return;
    }
    
    /* process RTCM file */
    if (!rtcm2rinex_procfile(&rtcm, rtcmfile)) {
        printf("RTCM file processing failed\n");
        free_rtcm(&rtcm);
        return;
    }
    
    /* set RINEX options */
    opt.rnxver=304; /* RINEX 3.04 */
    opt.obstype=OBSTYPE_PR|OBSTYPE_CP|OBSTYPE_DOP; /* pseudorange, carrier phase, doppler */
    opt.navsys=SYS_GPS|SYS_GLO; /* GPS and GLONASS only */
    opt.tint=30.0; /* 30-second interval */
    sprintf(opt.prog,"%s","RTCM2RINEX");
    sprintf(opt.runby,"%s","EXAMPLE");
    sprintf(opt.marker,"%s","MARKER");
    sprintf(opt.markerno,"%s","MARKER_NUMBER");
    
    /* convert to RINEX */
    if (!rtcm2rinex_convert(&rtcm, &opt, rinexfile)) {
        printf("RINEX conversion failed\n");
        free_rtcm(&rtcm);
        return;
    }
    
    printf("Converted to %s successfully with custom options\n", rinexfile);
    
    /* free resources */
    free_rtcm(&rtcm);
}

/* main ----------------------------------------------------------------------*/
int main(int argc, char **argv)
{
    if (argc<3) {
        printf("usage: example rtcm_file rinex_file\n");
        return -1;
    }
    
    /* run examples */
    example1(argv[1], argv[2]);
    
    char rinexfile2[256], rinexfile3[256];
    sprintf(rinexfile2, "%s.2", argv[2]);
    sprintf(rinexfile3, "%s.3", argv[2]);
    
    example2(argv[1], rinexfile2);
    example3(argv[1], rinexfile3);
    
    return 0;
} 