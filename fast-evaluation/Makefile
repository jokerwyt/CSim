#
# Student makefile for Cache Lab
# 
CC = gcc
CFLAGS = -g -Wall -Werror -std=c99 

.PHONY: all
all: tracegen

tracegen: tracegen.c trans.o cachelab.c
	$(CC) $(CFLAGS) -O0 -o tracegen tracegen.c trans.o cachelab.c -L. -lcsim -DDEBUG

trans.o: trans.c
	$(CC) $(CFLAGS) -O0 -c trans.c 

#
# Clean the src dirctory
#
clean:
	rm -rf *.o
	rm -f *.bc
	rm -f csim
	rm -f test-trans tracegen tracegen-ct
	rm -f trace.all trace.f*
	rm -f .csim_results .marker
