pragma circom 2.1.0;

include "../node_modules/circomlib/circuits/poseidon.circom";

// Verify a Merkle proof of membership for a tree of depth 16
template MerkleVerifier(depth) {
    // Public Inputs
    signal input root;
    signal input nullifierHash;

    // Private Inputs
    signal input secret;
    signal input pathElements[depth];
    signal input pathIndices[depth]; // 0 for left, 1 for right

    // Intermediate commitment hash
    signal commitment;
    component commitmentHasher = Poseidon(1);
    commitmentHasher.inputs[0] <== secret;
    commitment <== commitmentHasher.out;

    // Verify Merkle path
    component hashers[depth];
    signal currentHash[depth + 1];
    currentHash[0] <== commitment;

    for (var i = 0; i < depth; i++) {
        hashers[i] = Poseidon(2);
        
        // Multiplex path indices to compute left/right node hashing
        hashers[i].inputs[0] <== (1 - pathIndices[i]) * currentHash[i] + pathIndices[i] * pathElements[i];
        hashers[i].inputs[1] <== pathIndices[i] * currentHash[i] + (1 - pathIndices[i]) * pathElements[i];

        currentHash[i + 1] <== hashers[i].out;
    }

    // Assert computed root matches public root
    root === currentHash[depth];
}

component main {public [root, nullifierHash]} = MerkleVerifier(16);
