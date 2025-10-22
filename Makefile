# Makefile for PSLFS - Virtual File System
CC = gcc
CFLAGS = -Wall -Wno-deprecated-declarations -g
LDFLAGS =

# Targets
all: base fbase

base: basic.c ds.h partition.h fstest1.h
	$(CC) $(CFLAGS) -o base basic.c $(LDFLAGS)

fbase: fbase.c ds.h partition.h fstest1.h
	$(CC) $(CFLAGS) -o fbase fbase.c $(LDFLAGS)

clean:
	rm -f base fbase
	rm -f *.psl *.fol *.fil *.fs
	rm -f authent
	rm -f alpha alpha.txt

install: all
	@echo "Running base to initialize filesystem..."
	./base

run: fbase
	@echo "Starting PSLFS..."
	./fbase

.PHONY: all clean install run
