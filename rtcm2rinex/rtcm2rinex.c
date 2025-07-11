/*------------------------------------------------------------------------------
* rtcm2rinex.c : RTCM to RINEX converter API
*
* This is a simplified API for RTCM to RINEX conversion based on RTKLIB
*-----------------------------------------------------------------------------*/
#include "rtklib.h"

/* Initialize conversion context ------------------------------------------------
* initialize RTCM to RINEX conversion context
* args   : rtcm_t *rtcm      IO  RTCM control struct
* return : status (1:ok 0:error)
* notes  : this function must be called before starting data conversion
*-----------------------------------------------------------------------------*/
int rtcm2rinex_init(rtcm_t *rtcm)
{
    trace(3,"rtcm2rinex_init:\n");
    
    if (!rtcm) return 0;
    
    return init_rtcm(rtcm);
}

/* Process RTCM data -----------------------------------------------------------
* process RTCM data
* args   : rtcm_t *rtcm      IO  RTCM control struct
*          uint8_t data      I   RTCM data (1 byte)
* return : status (-1:error, 0:no message, 1:observation data, 2:nav data,
*                  3:station info, 9:debug)
* notes  : call this function repeatedly to process RTCM data stream
*-----------------------------------------------------------------------------*/
int rtcm2rinex_procdata(rtcm_t *rtcm, uint8_t data)
{
    trace(5,"rtcm2rinex_procdata: data=%02x\n",data);
    
    if (!rtcm) return -1;
    
    return input_rtcm3(rtcm, data);
}

/* Process RTCM file -----------------------------------------------------------
* process RTCM file
* args   : rtcm_t *rtcm      IO  RTCM control struct
*          const char *file  I   RTCM file path
* return : status (1:ok 0:error)
* notes  : this function reads an entire RTCM file and processes it
*-----------------------------------------------------------------------------*/
int rtcm2rinex_procfile(rtcm_t *rtcm, const char *file)
{
    FILE *fp;
    uint8_t data;
    int ret=0;
    
    trace(3,"rtcm2rinex_procfile: file=%s\n",file);
    
    if (!rtcm || !(fp=fopen(file,"rb"))) return 0;
    
    while (fread(&data,1,1,fp) == 1) {
        if ((ret=input_rtcm3(rtcm,data))) break;
    }
    
    fclose(fp);
    return ret;
}

/* Convert to RINEX -----------------------------------------------------------
* convert RTCM data to RINEX format
* args   : rtcm_t *rtcm      IO  RTCM control struct
*          rnxopt_t *opt     I   RINEX options
*          const char *file  I   output RINEX file path
* return : status (1:ok 0:error)
*-----------------------------------------------------------------------------*/
int rtcm2rinex_convert(rtcm_t *rtcm, rnxopt_t *opt, const char *outfile)
{
    FILE *fp;
    gtime_t time={0};
    int i,j;
    
    trace(3,"rtcm2rinex_convert: outfile=%s\n",outfile);
    
    if (!rtcm || !opt || !(fp=fopen(outfile,"w"))) return 0;
    
    /* output RINEX header */
    outrnxobsh(fp,opt,&rtcm->nav);
    
    /* output RINEX observation data */
    for (i=0;i<rtcm->obs.n;i=j) {
        /* extract observation data for a single epoch */
        for (j=i+1;j<rtcm->obs.n;j++) {
            if (timediff(rtcm->obs.data[j].time,rtcm->obs.data[i].time)>DTTOL) break;
        }
        
        /* output RINEX observation data for the epoch */
        outrnxobsb(fp,opt,rtcm->obs.data+i,j-i,0);
    }
    
    fclose(fp);
    return 1;
}

/* Simple conversion from RTCM file to RINEX file ------------------------------
* convert RTCM file to RINEX file with default options
* args   : const char *rtcmfile  I   input RTCM file path
*          const char *outfile   I   output RINEX file path
*          double ver            I   RINEX version (2.10,2.11,2.12,3.00,3.01,3.02,3.03,3.04)
* return : status (1:ok 0:error)
*-----------------------------------------------------------------------------*/
int rtcm2rinex_simple(const char *rtcmfile, const char *outfile, double ver)
{
    rnxopt_t opt={{0}};
    rtcm_t rtcm={0};
    char *ofile[9]={0};
    int ret;
    
    trace(3,"rtcm2rinex_simple: rtcmfile=%s outfile=%s ver=%.2f\n",rtcmfile,outfile,ver);
    
    /* initialize RTCM and RINEX options */
    if (!init_rtcm(&rtcm)) return 0;
    
    /* set RINEX options */
    opt.rnxver=ver*100.0; /* ver=3.04 -> rnxver=304 */
    opt.obstype=OBSTYPE_PR|OBSTYPE_CP;
    opt.navsys=SYS_GPS|SYS_GLO|SYS_GAL|SYS_QZS|SYS_SBS|SYS_CMP|SYS_IRN;
    sprintf(opt.prog,"%s","RTCM2RINEX");
    sprintf(opt.runby,"%s","RTKLIB");
    
    /* set output files */
    ofile[0]=(char *)outfile;
    
    /* convert file */
    ret=convrnx(STRFMT_RTCM3,&opt,rtcmfile,ofile);
    
    /* free resources */
    free_rtcm(&rtcm);
    
    return ret;
}

#ifdef TEST_MAIN
/* main function for testing --------------------------------------------------*/
int main(int argc, char **argv)
{
    if (argc<3) {
        fprintf(stderr,"usage: rtcm2rinex rtcm_file rinex_file [rinex_version]\n");
        return -1;
    }
    
    double ver=3.04; /* default RINEX version */
    if (argc>=4) ver=atof(argv[3]);
    
    if (!rtcm2rinex_simple(argv[1],argv[2],ver)) {
        fprintf(stderr,"conversion error\n");
        return -1;
    }
    
    return 0;
}
#endif /* TEST_MAIN */ 