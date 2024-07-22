#pragma once
#include <vector>
#include "src/utils.h"
using namespace std;

class Node {
  public:
    string path;
    // what data structure should we use to hold the children?
    vector<Node *> children;
    // bool isTerminal;
    
    // just hardcode the type of value in here for now
    // change the remove method as well, it doesn't make sense to 
    // delete a function pointer
    Handler value;

    bool isWildcard;
    string wildcardContent;

    Node(string path = "/", Handler value = nullptr);
    void addChild(Node *);
    void setValue(Handler);
    bool isTerminal();
    vector<Node *> &getChildren();
};

class Trie {
    Node *root;
    Node *_remove(Node *n, string targetPath, vector<string> &paths, int index);

  public:
    Trie(string root);
    Node *find(string path);
    void insert(string path, Handler handler);
    void remove(string path);
    void display(Node *n = nullptr);
};
