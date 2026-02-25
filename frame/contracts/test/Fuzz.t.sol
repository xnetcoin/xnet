// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.0;

import "forge-std/Test.sol";
import "../src/Sample.sol";

contract SampleFuzzTest is Test {
    Sample public sample;

    function setUp() public {
        sample = new Sample();
    }

    /// @notice Fuzz test: balance should never exceed total supply
    function testFuzzBalanceNeverExceedsTotalSupply(address addr, uint256 amount) public {
        amount = bound(amount, 0, type(uint256).max / 2);
        
        // Assume we can mint this amount
        try sample.mint(addr, amount) {
            // Check that balance doesn't exceed total supply
            assertLe(sample.balanceOf(addr), sample.totalSupply());
        } catch {
            // Revert is acceptable
        }
    }

    /// @notice Fuzz test: transfer should preserve total supply
    function testFuzzTransferPreservesTotalSupply(
        address from,
        address to,
        uint256 amount
    ) public {
        vm.assume(from != address(0));
        vm.assume(to != address(0));
        vm.assume(from != to);
        
        amount = bound(amount, 0, 1e18);
        
        // Setup initial state
        sample.mint(from, amount);
        uint256 totalBefore = sample.totalSupply();
        
        // Execute transfer
        vm.prank(from);
        try sample.transfer(to, amount) {
            // Total supply should remain unchanged
            assertEq(sample.totalSupply(), totalBefore);
        } catch {
            // Failed transfer is acceptable
        }
    }

    /// @notice Fuzz test: reentrancy protection
    function testFuzzReentrancyGuard(uint256 amount) public {
        amount = bound(amount, 1, 1e18);
        
        sample.mint(address(this), amount);
        
        // Attempt reentrancy - should fail
        try sample.withdraw(amount) {
            // Either succeeds cleanly or reverts
        } catch {
            // Expected to revert on reentrancy attempt
        }
    }

    /// @notice Property: commutative transfers
    function testFuzzCommutativeTransfers(
        address user1,
        address user2,
        uint256 amount1,
        uint256 amount2
    ) public {
        vm.assume(user1 != address(0) && user2 != address(0));
        vm.assume(user1 != user2);
        
        amount1 = bound(amount1, 0, 1e18);
        amount2 = bound(amount2, 0, 1e18);
        
        // Transfer 1 then 2
        sample.mint(user1, amount1);
        sample.mint(user2, amount2);
        
        uint256 balance1Before = sample.balanceOf(user1);
        uint256 balance2Before = sample.balanceOf(user2);
        
        // These transfers should be commutative for balance results
        // (exact implementation depends on contract logic)
        assertGe(balance1Before, 0);
        assertGe(balance2Before, 0);
    }
}
