// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.0;

import "forge-std/Test.sol";
import "../src/Sample.sol";

contract SampleSecurityTest is Test {
    Sample public sample;
    address public attacker;
    address public victim;

    function setUp() public {
        sample = new Sample();
        attacker = address(0xBAD);
        victim = address(0xDEAD);
    }

    /// @notice Test: reentrancy attack protection
    function testReentrancyAttackProtection() public {
        sample.mint(victim, 100 ether);
        
        vm.prank(victim);
        sample.approve(attacker, 50 ether);
        
        // Attacker tries to withdraw twice
        vm.prank(attacker);
        try sample.withdraw(50 ether) {
            // First withdrawal
            vm.prank(attacker);
            try sample.withdraw(50 ether) {
                fail("Reentrancy attack succeeded!");
            } catch {
                // Expected - reentrancy guard should prevent this
            }
        } catch {
            // Withdrawal rejected
        }
    }

    /// @notice Test: integer overflow protection
    function testIntegerOverflowProtection() public {
        sample.mint(victim, type(uint256).max);
        
        // Attempt to mint more - should revert
        vm.expectRevert();
        sample.mint(victim, 1);
    }

    /// @notice Test: unauthorized access
    function testUnauthorizedAccess() public {
        vm.prank(victim);
        sample.mint(victim, 100 ether);
        
        // Attacker tries to withdraw without approval
        vm.prank(attacker);
        vm.expectRevert();
        sample.withdraw(50 ether);
    }

    /// @notice Test: malicious contract interaction
    function testMaliciousContractInteraction() public {
        // Create a malicious contract that tries to reenenter
        MaliciousContract malicious = new MaliciousContract(address(sample));
        
        sample.mint(address(malicious), 100 ether);
        
        // Attempt malicious interaction
        vm.expectRevert();
        malicious.attack();
    }

    /// @notice Test: flash loan protection (if applicable)
    function testFlashLoanProtection() public {
        sample.mint(address(this), 100 ether);
        
        // Attempt flash loan-like behavior
        // (exact implementation depends on contract)
        uint256 balanceBefore = sample.balanceOf(address(this));
        
        try sample.withdraw(100 ether) {
            // Balance should be correctly updated
            assertEq(sample.balanceOf(address(this)), balanceBefore - 100 ether);
        } catch {
            // Revert is acceptable
        }
    }
}

/// @notice Malicious contract for testing reentrancy
contract MaliciousContract {
    Sample public sample;
    uint256 public reentrancyCount = 0;

    constructor(address _sample) {
        sample = Sample(_sample);
    }

    function attack() public {
        sample.withdraw(50 ether);
    }

    receive() external payable {
        reentrancyCount++;
        if (reentrancyCount < 3) {
            // Attempt reentrancy
            sample.withdraw(50 ether);
        }
    }
}
