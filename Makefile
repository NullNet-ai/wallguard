SHELL := /bin/sh

PROJECT_NAME = wallmon
VERSION = 0.1.0

EXEC_RELEASE = $(PROJECT_NAME)
EXEC_DEBUG = $(PROJECT_NAME)_d
EXEC_TEST = $(PROJECT_NAME)_test

CC = gcc
CFLAGS = -Wall -Wextra -I./src -DVERSION=\"$(VERSION)\"
LDFLAGS = -lssl -lcrypto -lcurl -l:libconfig.a

UNAME_S := $(shell uname -s)

DEBUG_CFLAGS = -g
RELEASE_CFLAGS = -O3

SRCS = $(shell find src -type f -name '*.c')
OBJS = $(patsubst src/%.c, src/%.o, $(filter-out src/main.c, $(SRCS)))
MAIN_OBJ = src/main.o

TEST_SRCS = $(shell find tests -type f -name '*.c')
TEST_OBJS = $(patsubst tests/%.c, tests/%.o, $(TEST_SRCS))

debug: CFLAGS += $(DEBUG_CFLAGS)
debug: EXECUTABLE = $(EXEC_DEBUG)

release: CFLAGS += $(RELEASE_CFLAGS)
release: EXECUTABLE = $(EXEC_RELEASE)

debug release: $(OBJS) $(MAIN_OBJ)
	$(CC) $(CFLAGS) -o $(EXECUTABLE) $(OBJS) $(MAIN_OBJ) $(LDFLAGS)


test: CFLAGS += -I/usr/include/CUnit -g
test: LDFLAGS += -lcunit
test: EXECUTABLE = $(EXEC_TEST)

test: $(OBJS) $(TEST_OBJS)
	$(CC) $(CFLAGS) -o $(EXECUTABLE) $(OBJS) $(TEST_OBJS) $(LDFLAGS)

clean:
	rm -f $(EXEC_TEST) $(EXEC_DEBUG) $(EXEC_RELEASE) $(OBJS) $(MAIN_OBJ) $(TEST_OBJS) 

.PHONY: release debug test clean
