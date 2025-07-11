# Makefile for RTCM to RINEX conversion tool

CC = gcc
CFLAGS = -O3 -DTRACE -DENAGLO -DENAQZS -DENAGAL -DENACMP -DENAIRN -DNFREQ=5 -DNEXOBS=3 -g
LDLIBS = -lm

SRC = .
INCLUDE = -I$(SRC)

OBJS = convbin.o rtkcmn.o rinex.o sbas.o preceph.o rcvraw.o convrnx.o \
       rtcm.o rtcm2.o rtcm3.o rtcm3e.o pntpos.o ephemeris.o ionex.o \
       stream.o streamsvr.o solution.o \
       novatel.o ss2.o ublox.o crescent.o skytraq.o javad.o nvs.o \
       binex.o rt17.o septentrio.o

LIB_OBJS = rtcm2rinex.o

all: rtcm2rinex example

rtcm2rinex: $(OBJS)
	$(CC) -o $@ $(OBJS) $(LDLIBS)

example: example.o $(LIB_OBJS) $(OBJS)
	$(CC) -o $@ example.o $(LIB_OBJS) $(OBJS) $(LDLIBS)

test_lib: rtcm2rinex.o
	$(CC) -o rtcm2rinex_lib rtcm2rinex.o -DTEST_MAIN $(OBJS) $(LDLIBS)

example.o: example.c rtcm2rinex.h
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

rtcm2rinex.o: rtcm2rinex.c rtcm2rinex.h
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

convbin.o: $(SRC)/convbin.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

rtkcmn.o: $(SRC)/rtkcmn.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

rinex.o: $(SRC)/rinex.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

sbas.o: $(SRC)/sbas.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

preceph.o: $(SRC)/preceph.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

rcvraw.o: $(SRC)/rcvraw.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

convrnx.o: $(SRC)/convrnx.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

rtcm.o: $(SRC)/rtcm.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

rtcm2.o: $(SRC)/rtcm2.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

rtcm3.o: $(SRC)/rtcm3.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

rtcm3e.o: $(SRC)/rtcm3e.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

pntpos.o: $(SRC)/pntpos.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

ephemeris.o: $(SRC)/ephemeris.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

ionex.o: $(SRC)/ionex.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

stream.o: $(SRC)/stream.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

streamsvr.o: $(SRC)/streamsvr.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

solution.o: $(SRC)/solution.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

novatel.o: $(SRC)/rcv/novatel.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

ss2.o: $(SRC)/rcv/ss2.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

ublox.o: $(SRC)/rcv/ublox.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

crescent.o: $(SRC)/rcv/crescent.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

skytraq.o: $(SRC)/rcv/skytraq.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

javad.o: $(SRC)/rcv/javad.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

nvs.o: $(SRC)/rcv/nvs.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

binex.o: $(SRC)/rcv/binex.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

rt17.o: $(SRC)/rcv/rt17.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

septentrio.o: $(SRC)/rcv/septentrio.c
	$(CC) -c $(CFLAGS) $(INCLUDE) $<

clean:
	rm -f rtcm2rinex rtcm2rinex_lib example *.o *.obs *.nav *.gnav *.hnav *.qnav *.sbs *.stackdump 