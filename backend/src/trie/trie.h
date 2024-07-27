#pragma once
#include <vector>
#include "src/http.h"

using std::string;
using RouteHandler = void (*)(http_request &, HTTP &);

class Node {
  public:
    string path;
    // what data structure should we use to hold the children?
    std::vector<Node *> children;
    // bool isTerminal;
    
    // just hardcode the type of value in here for now
    // change the remove method as well, it doesn't make sense to 
    // delete a function pointer
    RouteHandler value;

    bool isWildcard;
    string wildcardContent;

    Node(string path = "/", RouteHandler value = nullptr);
    void addChild(Node *);
    void setValue(RouteHandler);
    bool isTerminal();
    std::vector<Node *> &getChildren();
};

class Trie {
    Node *root;
    Node *_remove(Node *n, string targetPath, std::vector<string> &paths, int index);

  public:
    Trie(string root);
    Node *find(string path);
    void insert(string path, RouteHandler handler);
    void remove(string path);
    void display(Node *n = nullptr);
};
