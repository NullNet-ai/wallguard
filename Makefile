CC = gcc
CFLAGS = -Iinclude

DEBUG_CFLAGS = -Wall -Wextra -g
RELEASE_CFLAGS = -Wall -Wextra -O3

SRCS = $(shell find src -type f -name '*.c')
OBJS = $(SRCS:.c=.o)

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
