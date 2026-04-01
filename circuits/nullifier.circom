pragma circom 2.1.0;

include "../node_modules/circomlib/circuits/poseidon.circom";

template NullifierHasher() {
    // Private Input
    signal input secret;
    // Public Input
    signal input appId;

    // Public Output
    signal output nullifierHash;

    component poseidon = Poseidon(2);
    poseidon.inputs[0] <== secret;
    poseidon.inputs[1] <== appId;

    nullifierHash <== poseidon.out;
}

component main {public [appId]} = NullifierHasher();
