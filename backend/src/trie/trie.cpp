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
void Trie::insert(string path, void *handler)
{
    if (path == "/") {
        // TODO
    }

    auto path_segments = utils::split_str(path, "/");

    Node *currentNode = this->root;
    for (auto p : path_segments) {
        bool pathAlreadyExits = false;

        for (auto c : currentNode->getChildren()) {
            if (c->path == p) {
                currentNode = c;
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

    currentNode->setValue(handler);
}

Node *Trie::find(string path)
{
    auto path_segments = utils::split_str(path, "/");

    Node *currentNode = this->root;
    for (auto p : path_segments) {
        bool found = false;

        for (auto c : currentNode->getChildren()) {
            if (c->path == p && c->isTerminal()) {
                currentNode = c;
                found = true;
                break;
            }
        }
        if (!found) {
            return nullptr;
        }
    }

    return currentNode;
}

// find a cleaner way of doing this without having to use targetPath
Node *Trie::_remove(Node *n, string targetPath, vector<string> &paths,
                    int index)
{
    // if (index-1 >= paths.size()) {
    //     std::cout << "index: " << index << endl;
    //     std::cout << n->path << endl;
    //     return nullptr;
    // }

    if (n == nullptr) {
        return nullptr;
    }

    if (n->path == targetPath) {
        if (!(n->isTerminal())) {
            return nullptr;
        }

        if (!(n->getChildren().empty())) {
            n->setValue(nullptr);
        }
        else {
            delete n;
            return n;
        }
    }
    else {
        Node *removed = nullptr;
        auto children = n->getChildren();
        for (int i = 0; i < children.size(); i++) {
            if (children[i]->path == paths[index]) {
                removed =
                    this->_remove(children[i], targetPath, paths, index + 1);
                if (removed) {
                    // why can't we not do: children.erase(...) ??
                    // children is just a reference to the original children, but for some reason
                    // the changes are not reflected in the original one
                    n->children.erase(n->children.begin() + i);
                }
                
                if (n->children.empty() && !n->isTerminal()) {
                    delete n;
                    return n;
                }
                break;
            }
        }
    }

    return nullptr;
}

void Trie::remove(string path)
{
    auto paths = utils::split_str(path, "/");
    auto target = paths.back();
    _remove(this->root, target, paths, 0);
}

void Trie::display(Node *n)
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

bool Node::isTerminal()
{
    if (this->handler) {
        return true;
    }

    return false;
}

void Node::addChild(Node *n)
{
    this->children.push_back(n);
}

vector<Node *> &Node::getChildren()
{
    return this->children;
}

void Node::setValue(void *n)
{
    this->handler = n;
}
