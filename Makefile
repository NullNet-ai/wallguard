SHELL := /bin/sh

CC = gcc
CFLAGS = -I./src
LDFLAGS = -lssl -lcrypto -lxml2

UNAME_S := $(shell uname -s)

ifeq ($(UNAME_S),FreeBSD)
    CFLAGS += -I/usr/local/include/libxml2
else
    CFLAGS += -I/usr/include/libxml2
endif

DEBUG_CFLAGS = -Wall -Wextra -g
RELEASE_CFLAGS = -Wall -Wextra -O3

SRCS = $(shell find src -type f -name '*.c')
OBJS = $(patsubst src/%.c, src/%.o, $(SRCS))

# Name of the produced artifact
EXEC = wallmon

# Debug build by default
all: debug

release: CFLAGS += $(RELEASE_CFLAGS)
release: $(EXEC)

debug: CFLAGS += $(DEBUG_CFLAGS)
debug: $(EXEC)

$(EXEC): $(OBJS)
	$(CC) $(CFLAGS) -o $(EXEC) $(OBJS) $(LDFLAGS)

src/%.o: src/%.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(EXEC) $(OBJS)
