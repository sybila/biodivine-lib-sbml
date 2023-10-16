// Rules for treating namespaces of detached elements:
//
// 1. When an element tree is detached, we copy all the namespace declarations to the tree root,
// meaning the original context is preserved.
//     Question: do we also copy the declarations that are not used? Probably not. If a child
//     is added with its own namespace, it will be transferred to us.
//
// 2. When an element tree is attached, we go through all the namespace declarations in the tree
// root, and we try to move them to the root of the parent tree. Either:
//     - The namespace does not exist in the root, in which case we can add it.
//     - The namespace exists with the same prefix, in which case we can ignore it.
//     - The namespace exists with different prefix, in which case we declare both prefixes.
//     - There is another namespace with the same prefix in the root or somewhere on the path
//     from root to the attached element. In such case, we keep the declaration on the child
//     element.
//
// For each namespace, we have a static structure where we keep the URL and a "default prefix".
// This prefix is the one that is used when creating new tags with this namespace. However, one
// can override this and use a different prefix if desired.
//
// Finally, there should be a "normalization" algorithm that goes through the whole document
// and tries to (a) remove unused namespace declarations (b) move declarations that can be moved
// as high in the element tree as possible, and (c) merge redundant declarations by renaming
// other prefixes. For "default" namespaces, these are mostly kept where they are, because any
// change has a significant impact on the document. Specifically, we won't promote a namespace
// to default or create a new prefix for a default namespace.
//
