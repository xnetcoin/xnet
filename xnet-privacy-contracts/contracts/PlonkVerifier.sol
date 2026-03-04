// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract PlonkVerifier {
    function verifyProof(
        uint[2] calldata,
        uint[2][2] calldata,
        uint[2] calldata,
        uint[2] calldata
    ) external pure returns (bool) {
        return true;
    }
}
