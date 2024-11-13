#include "internal.h"

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static err_t t_memdup(const void *ptr, size_t size, void **dest) {
  void *const result = malloc(size);
  if (result) {
    memcpy(result, ptr, size);
    *dest = result;
    return false;
  }
  return true;
}

static err_t t_strdup(const char *str, char **dest) {
  return t_memdup(str, strlen(str) + 1, (void **)dest);
}

typedef struct Node {
  char *key;
  void *value;
  void (*deleteValue)(void *value);
  struct Node *left;
  struct Node *right;
  int height;
} *Node;

struct TMap {
  struct Node *root;
};

static int int_max(int self, int other) { return self > other ? self : other; }

static err_t createNode(const char *key, void *value,
                        void (*deleteValue)(void *value), Node *out) {
  Node node = (struct Node *)malloc(sizeof(struct Node));
  if (!node) {
    return true;
  }
  if (t_strdup(key, &node->key)) {
    free(node);
    return true;
  }
  node->value = value;
  node->deleteValue = deleteValue;
  node->left = node->right = NULL;
  node->height = 1;
  *out = node;
  return false;
}

static int getHeight(Node node) { return node == NULL ? 0 : node->height; }

static int getBalanceFactor(Node node) {
  return node == NULL ? 0 : getHeight(node->left) - getHeight(node->right);
}

static Node rotateRight(Node y) {
  Node x = y->left;
  Node T2 = x->right;

  x->right = y;
  y->left = T2;

  y->height = 1 + int_max(getHeight(y->left), getHeight(y->right));
  x->height = 1 + int_max(getHeight(x->left), getHeight(x->right));

  return x;
}

static Node rotateLeft(Node x) {
  Node y = x->right;
  Node T2 = y->left;

  y->left = x;
  x->right = T2;

  y->height = 1 + int_max(getHeight(y->left), getHeight(y->right));
  x->height = 1 + int_max(getHeight(x->left), getHeight(x->right));

  return y;
}

static Node balanceNode(Node node) {
  int balance = getBalanceFactor(node);

  if (balance > 1) {
    if (getBalanceFactor(node->left) < 0) {
      node->left = rotateLeft(node->left);
    }
    return rotateRight(node);
  }

  if (balance < -1) {
    if (getBalanceFactor(node->right) > 0) {
      node->right = rotateRight(node->right);
    }
    return rotateLeft(node);
  }

  return node;
}

static err_t insertNode(Node node, const char *key, void *value,
                        void (*deleteValue)(void *value), Node *out) {
  if (node == NULL) {
    return createNode(key, value, deleteValue, out);
  }

  int leading_same = 0;
  while (node->key[leading_same] != '\0' && key[leading_same] != '\0' &&
         node->key[leading_same] == key[leading_same]) {
    leading_same++;
  }
  const int cmp = strcmp(node->key + leading_same, key + leading_same);

  if (cmp > 0) {
    if (insertNode(node->left, key, value, deleteValue, &node->left)) {
      return true;
    }
  } else if (cmp < 0) {
    if (insertNode(node->right, key, value, deleteValue, &node->right)) {
      return true;
    }
  } else {
    return true;
  }

  node->height = 1 + int_max(getHeight(node->left), getHeight(node->right));

  *out = balanceNode(node);
  return false;
}

TMap TMap_new(void) {
  const TMap result = malloc(sizeof(struct TMap));
  if (result) {
    result->root = NULL;
  }
  return result;
}

err_t TMap_insert(TMap map, const char *key, void *value,
                  void (*deleteValue)(void *value)) {
  return insertNode(map->root, key, value, deleteValue, &map->root);
}

void *TMap_search(TMap map, const char *key) {
  Node current = map->root;
  int leading_same = 0;
  while (current != NULL) {
    while (current->key[leading_same] != '\0' && key[leading_same] != '\0' &&
           current->key[leading_same] == key[leading_same]) {
      leading_same++;
    }
    const int cmp = strcmp(current->key + leading_same, key + leading_same);

    if (cmp == 0) {
      return current->value;
    } else if (cmp > 0) {
      current = current->left;
    } else {
      current = current->right;
    }
  }
  return NULL;
}

bool TMap_has(TMap map, const char *key) {
  Node current = map->root;
  int leading_same = 0;
  while (current != NULL) {
    while (current->key[leading_same] != '\0' && key[leading_same] != '\0' &&
           current->key[leading_same] == key[leading_same]) {
      leading_same++;
    }
    const int cmp = strcmp(current->key + leading_same, key + leading_same);

    if (cmp == 0) {
      return true;
    } else if (cmp > 0) {
      current = current->left;
    } else {
      current = current->right;
    }
  }
  return false;
}

static void deleteNode(Node node) {
  if (!node) {
    return;
  }
  deleteNode(node->left);
  deleteNode(node->right);
  if (node->deleteValue) {
    node->deleteValue(node->value);
  }
  free(node->key);
  free(node);
}

void TMap_delete(TMap self) {
  deleteNode(self->root);
  free(self);
}
