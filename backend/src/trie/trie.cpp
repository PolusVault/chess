#include "trie.h"
#include "src/utils.h"
#include <iostream>

using namespace std;

// Naive implementation of Trie

Trie::Trie(string root)
{
    this->root = new Node(root);
}

// TODO: do this recursively
void Trie::insert(string path, RouteHandler value)
{
    if (path == "/") {
        this->root->setValue(value);
        return;
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
        if (p=="*") {
            newNode->isWildcard = true;
        }
        currentNode->addChild(newNode);
        currentNode = newNode;
    }

    currentNode->setValue(value);
}

Node *Trie::find(string path)
{
    if (path == "/") {
        if (!this->root->isTerminal()) {
            return nullptr;
        }
        return this->root;
    }

    auto path_segments = utils::split_str(path, "/");

    Node *currentNode = this->root;
    for (auto p : path_segments) {
        bool found = false;

        for (auto c : currentNode->getChildren()) {
            // wildcards only work for the ending path for now
            if (c->path == p || (c->isWildcard && p == path_segments.back())) {
                if (c->isWildcard) {
                    c->wildcardContent = p; 
                }
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
                    // children is just a reference to the original children,
                    // but for some reason the changes are not reflected in the
                    // original one
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

// we won't ever need to remove anything in our use case, I just wanted to
// do this function for personal future reference 
void Trie::remove(string path)
{
    if (path == "/") {
        vector<string> paths = {"/"};
        _remove(this->root, path, paths, 0);
    }
    else {
        auto paths = utils::split_str(path, "/");
        auto target = paths.back();
        _remove(this->root, target, paths, 0);
    }
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

Node::Node(string path, RouteHandler value)
{
    this->path = path;
    this->value = value;
}

bool Node::isTerminal()
{
    if (this->value) {
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

void Node::setValue(RouteHandler n)
{
    this->value = n;
}
