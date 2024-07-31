SHELL := /bin/sh

CC = gcc
CFLAGS = -I./src

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
	$(CC) $(CFLAGS) -o $(EXEC) $(OBJS)

src/%.o: src/%.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(EXEC) $(OBJS)

# @TODO
test: $(EXEC)
	@echo "Running tests..."
	@$(EXEC) --test
