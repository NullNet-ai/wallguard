#ifndef _LINKED_LIST_H_
#define _LINKED_LIST_H_

#include <utils/common.h>

struct llist {
    struct llist* next;
    void*         data;
};

typedef struct llist llist_t;

/**
 * @brief Creates a new linked list node with the given data.
 *
 * @param data A pointer to the data to store in the new node.
 * @return A pointer to the newly created node.
 */
llist_t* ll_create_node(void* data);

/**
 * @brief Adds a new node with the specified data to the front of the linked list.
 *
 * @param head Pointer to the pointer to the head of the list.
 * @param data A pointer to the data to store in the new node.
 */
void ll_push_front(llist_t** head, void* data);

/**
 * @brief Adds a new node with the specified data to the end of the linked list.
 *
 * @param head Pointer to the pointer to the head of the list.
 * @param data A pointer to the data to store in the new node.
 */
void ll_push_back(llist_t** head, void* data);

/**
 * @brief Frees all nodes in the linked list, releasing allocated memory.
 *
 * @param head Pointer to the head of the list.
 */
void ll_free(llist_t* list);

/**
 * @brief Calculates the length of a linked list.
 *
 * @param list Pointer to the head node of the linked list.
 *
 * @return The number of nodes in the linked list.
 */
size_t ll_length(llist_t* list);

/**
 * @brief Iterates over a linked list.
 */
#define LL_FOREACH(LIST, ITEM) for (llist_t* ITEM = LIST; ITEM != NULL; ITEM = ITEM->next)

#endif
