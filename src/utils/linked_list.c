#include <utils/linked_list.h>

llist_t* ll_create_node(void* data) {
    llist_t* node = W_MALLOC(sizeof(llist_t));

    node->data = data;
    node->next = NULL;

    return node;
}

void ll_push_front(llist_t** head, void* data) {
    llist_t* node = ll_create_node(data);

    node->next = *head;

    *head = node;
}

void ll_push_back(llist_t** head, void* data) {
    llist_t* temp = *head;

    while (temp->next) {
        temp = temp->next;
    }

    temp->next = ll_create_node(data);
}

void ll_free(llist_t* head) {
    while (head != NULL) {
        llist_t* temp = head;

        head = head->next;

        W_FREE(temp);
    }
}

size_t ll_length(llist_t* list) {
    size_t retval = 0;

    for (llist_t* temp = list; temp != NULL; temp = temp->next) {
        ++retval;
    }

    return retval;
}
