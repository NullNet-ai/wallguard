SHELL := /bin/sh

# Compiler and flags
CC = gcc
CFLAGS = -Wall -Wextra -I./src
LDFLAGS = -lssl -lcrypto -lxml2

# Platform-specific flags
UNAME_S := $(shell uname -s)

ifeq ($(UNAME_S),FreeBSD)
    CFLAGS += -I/usr/local/include/libxml2
else
    CFLAGS += -I/usr/include/libxml2
endif

# Build type flags
DEBUG_CFLAGS = -g
RELEASE_CFLAGS = -O3

# Source and object files
SRCS = $(shell find src -type f -name '*.c')
OBJS = $(patsubst src/%.c, src/%.o, $(filter-out src/main.c, $(SRCS)))

# Executables
EXEC = wallmon
EXEC_TEST = $(EXEC)_test

# Default target
all: debug

# Release build
release: CFLAGS += $(RELEASE_CFLAGS)
release: $(EXEC)

# Debug build
debug: CFLAGS += $(DEBUG_CFLAGS)
debug: $(EXEC)

# Link the main executable
$(EXEC): $(OBJS) src/main.o
	$(CC) $(CFLAGS) -o $(EXEC) $(OBJS) src/main.o $(LDFLAGS)

# Test source and object files
TEST_SRCS = $(shell find tests -type f -name '*.c')
TEST_OBJS = $(patsubst tests/%.c, tests/%.o, $(TEST_SRCS))

# Test build
# TODO: Test on a FreeBSD system
test: CFLAGS += -I/usr/include/CUnit
test: LDFLAGS += -lcunit
test: $(EXEC_TEST)

# Link the test executable
$(EXEC_TEST): $(OBJS) $(TEST_OBJS)
	$(CC) $(CFLAGS) -o $(EXEC_TEST) $(OBJS) $(TEST_OBJS) $(LDFLAGS)

# Clean up generated files
clean:
	rm -f $(EXEC) $(EXEC_TEST) $(OBJS) $(TEST_OBJS)

.PHONY: all release debug test clean
