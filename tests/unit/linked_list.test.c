#include "linked_list.test.h"
#include "utils/linked_list.h"

#include <CUnit/CUnit.h>
#include <CUnit/Basic.h>

void test_ll_create_node() {
    int      data = 42;
    llist_t* node = ll_create_node(&data);

    CU_ASSERT_PTR_NOT_NULL_FATAL(node);
    CU_ASSERT_EQUAL(node->data, &data);
    CU_ASSERT_PTR_NULL(node->next);

    W_FREE(node);
}

void test_ll_push_front() {
    int      data1 = 42, data2 = 84;
    llist_t* head = NULL;

    ll_push_front(&head, &data1);
    CU_ASSERT_PTR_NOT_NULL_FATAL(head);
    CU_ASSERT_EQUAL(head->data, &data1);
    CU_ASSERT_PTR_NULL(head->next);

    ll_push_front(&head, &data2);
    CU_ASSERT_PTR_NOT_NULL_FATAL(head);
    CU_ASSERT_EQUAL(head->data, &data2);
    CU_ASSERT_PTR_NOT_NULL(head->next);
    CU_ASSERT_EQUAL(head->next->data, &data1);

    ll_free(head);
}

void test_ll_push_back() {
    int      data1 = 42, data2 = 84;
    llist_t* head = ll_create_node(&data1);

    ll_push_back(&head, &data2);
    CU_ASSERT_PTR_NOT_NULL_FATAL(head->next);
    CU_ASSERT_EQUAL(head->next->data, &data2);
    CU_ASSERT_PTR_NULL(head->next->next);

    ll_free(head);
}

void test_ll_length() {
    int      data1 = 42, data2 = 84, data3 = 168;
    llist_t* head = NULL;

    CU_ASSERT_EQUAL(ll_length(head), 0);

    ll_push_front(&head, &data1);
    CU_ASSERT_EQUAL(ll_length(head), 1);

    ll_push_front(&head, &data2);
    CU_ASSERT_EQUAL(ll_length(head), 2);

    ll_push_front(&head, &data3);
    CU_ASSERT_EQUAL(ll_length(head), 3);

    ll_free(head);
}

void test_ll_free() {
    int      data1 = 42, data2 = 84;
    llist_t* head = NULL;

    ll_push_front(&head, &data1);
    ll_push_front(&head, &data2);

    ll_free(head);
    CU_PASS("List freed without issues");
}

void add_linked_list_tests(void) {
    CU_pSuite suite = CU_add_suite("Linked List Tests", NULL, NULL);
    CU_add_test(suite, "test_ll_create_node", test_ll_create_node);
    CU_add_test(suite, "test_ll_push_front", test_ll_push_front);
    CU_add_test(suite, "test_ll_push_back", test_ll_push_back);
    CU_add_test(suite, "test_ll_length", test_ll_length);
    CU_add_test(suite, "test_ll_free", test_ll_free);
}