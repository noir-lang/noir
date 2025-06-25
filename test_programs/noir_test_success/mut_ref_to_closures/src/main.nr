// Test for mutable references to closures and functions
// This addresses issue #8478: https://github.com/noir-lang/noir/issues/8478

// Test 1: Passing a function as a parameter and returning a boolean
fn test_func_as_param(fg: fn(u8) -> bool) -> bool {
    // Compare results when calling the function with different values
    fg(0) != fg(1)
}

// Test 2: Updating a function reference through a mutable reference
fn update_func_ref(f: &mut (fn(u8) -> bool), g: fn(u8) -> bool) {
    *f = g;
}

// Named functions for testing
fn is_zero(x: u8) -> bool {
    x == 0
}

fn is_one(x: u8) -> bool {
    x == 1
}

// Test 3: Main test using function references with control flow
fn test_function_references(b: bool) -> bool {
    // Start with is_zero function
    let mut f = is_zero;
    
    // Verify initial state
    assert(f(0) == true);
    assert(f(1) == false);
    assert(test_func_as_param(f) == true);
    
    // Update function reference based on parameter
    if b {
        // This should keep the same function (testing self-assignment)
        update_func_ref(&mut f, f);
    } else {
        // This should change the function
        update_func_ref(&mut f, is_one);
    }
    
    // Final check depends on which branch was taken
    if b {
        // Function should still be is_zero
        assert(f(0) == true);
        assert(f(1) == false);
        true
    } else {
        // Function should now be is_one
        assert(f(0) == false);
        assert(f(1) == true);
        false
    }
}

// Test 4: Testing closures stored in mutable references
fn test_closure_references() -> bool {
    // Create closures
    let value_a = 42;
    let closure_a = |x: u8| x as Field == value_a;
    
    let value_b = 100;
    let closure_b = |x: u8| x as Field == value_b;
    
    // Store in mutable reference
    let mut f = closure_a;
    
    // Check initial state
    assert(f(42) == true);
    assert(f(100) == false);
    
    // Update through reference
    update_closure_ref(&mut f, closure_b);
    
    // Check updated state
    assert(f(42) == false);
    assert(f(100) == true);
    
    true
}

// Helper to update closure reference
fn update_closure_ref(f: &mut (fn(u8) -> bool), g: fn(u8) -> bool) {
    *f = g;
}

// Test 5: Complex test with nested closures and function references
fn test_nested_closures() -> bool {
    let outer_value = 10;
    
    // Create a closure that returns another closure
    let create_checker = |threshold: Field| {
        |x: u8| (x as Field + outer_value) > threshold
    };
    
    // Create different checker functions with different thresholds
    let low_checker = create_checker(15);
    let high_checker = create_checker(25);
    
    // Store in mutable reference
    let mut current_checker = low_checker;
    
    // Test initial state (10 + 6 = 16 > 15)
    assert(current_checker(6) == true);
    // Test initial state (10 + 4 = 14 < 15)
    assert(current_checker(4) == false);
    
    // Update the reference
    update_func_ref(&mut current_checker, high_checker);
    
    // Test updated state (10 + 16 = 26 > 25)
    assert(current_checker(16) == true);
    // Test updated state (10 + 14 = 24 < 25)
    assert(current_checker(14) == false);
    
    true
}

// Main function that runs all tests
fn main() -> pub bool {
    assert(test_function_references(true) == true);
    assert(test_function_references(false) == false);
    assert(test_closure_references() == true);
    assert(test_nested_closures() == true);
    
    true
} 
