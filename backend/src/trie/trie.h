#pragma once
#include <vector>
using namespace std;

class Node {
    void* handler;
    
  public:
    string path;
    // what data structure should we use to hold the children?
    vector<Node*> children; 
    // bool isTerminal;

    Node(string path = "/", void* handler = nullptr);
    void addChild(Node*);
    void setValue(void*);
    bool isTerminal();
    vector<Node*>& getChildren();
};

class Trie {
    Node* root;
    Node* _remove(Node* n, string targetPath, vector<string>& paths, int index);
  public:
    Trie(string root);
    Node* find(string path);
    void insert(string path, void* handler);
    void remove(string path);
    void display(Node* n = nullptr);
};

