// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract PoseidonHasher {
    function poseidon(bytes32[2] calldata inputs)
        external pure returns (bytes32)
    {
        return keccak256(abi.encode(inputs[0], inputs[1]));
    }
}
