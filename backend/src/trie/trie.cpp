#include "trie.h"
#include "src/utils.h"
#include <iostream>

using namespace std;

// Naive implementation of Trie

Trie::Trie(string root)
{
    this->root = new Node();
}

// TODO: do this recursively
void Trie::insert(string path)
{
    if (path == "/") {
        // TODO
    }

    auto path_segments = utils::split_str(path, "/");

    Node *currentNode = this->root;
    for (auto p : path_segments) {
        bool pathAlreadyExits = false;
        
        // this doesn't seem right?
        for (auto c : currentNode->getChildren()) {
            if (c->path == p) {
                pathAlreadyExits = true;
                break;
            }
        }

        if (pathAlreadyExits)
            continue;

        auto newNode = new Node(p);
        currentNode->addChild(newNode);
        currentNode = newNode;
    }
}

void Trie::find(string path) {}

void Trie::remove(string path) {}

void Trie::display(Node* n)
{
    Node *currentNode = n;

    if (!n) {
        currentNode = this->root;
    }

    std::cout << currentNode->path << std::endl;

    for (auto c : currentNode->getChildren()) {
        this->display(c);
    }
}

Node::Node(string path, void *handler)
{
    this->path = path;
    this->handler = handler;
}

void Node::addChild(Node *n)
{
    this->children.push_back(n);
}

vector<Node *> Node::getChildren()
{
    return this->children;
}
