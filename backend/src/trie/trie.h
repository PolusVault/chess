#pragma once
#include <vector>
using namespace std;

class Node {
    void* handler;
    bool isTerminal;
    vector<Node*> children; 
    
  public:
    string path;

    Node(string path = "/", void* handler = nullptr);
    void addChild(Node*);
    vector<Node*> getChildren();
};

class Trie {
    Node* root;
  public:
    Trie(string root);
    void find(string path);
    void insert(string path);
    void remove(string path);
    void display(Node* n = nullptr);
};

