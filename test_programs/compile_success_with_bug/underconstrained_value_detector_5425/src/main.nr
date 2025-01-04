unconstrained fn maximum_price(options: [u32; 3]) -> u32 {
    let mut maximum_option = 0;
    for option in options {
        if (option > maximum_option) {
            maximum_option = option;
        }
    }
    maximum_option
}

fn main(sandwiches: pub [u32; 3], drinks: pub [u32; 3], snacks: pub [u32; 3], best_value: u32) {
    unsafe {
        //@safety: testing context
        let meal_deal_cost: u32 = 390;
        let most_expensive_sandwich = maximum_price(sandwiches);
        let mut sandwich_exists = false;
        for sandwich_price in sandwiches {
            assert(sandwich_price <= most_expensive_sandwich);
            sandwich_exists |= sandwich_price == most_expensive_sandwich;
        }

        // maximum_price(sandwiches) is properly constrained with this assert,
        // linking the result to the arguments
        assert(sandwich_exists);

        let most_expensive_drink = maximum_price(drinks);
        let mut drink_exists = false;
        for drink_price in drinks {
            assert(drink_price <= most_expensive_drink);
            drink_exists |= drink_price == most_expensive_drink;
        }

        // maximum_price(drinks) is properly constrained with this assert,
        // linking the result to the arguments
        assert(drink_exists);

        let most_expensive_snack = maximum_price(snacks);
        // maximum_price(snacks)'s result isn't constrained against `snacks`
        // in any way, triggering the missing Brillig constraint check

        assert(
            best_value
                == (
                    most_expensive_sandwich + most_expensive_drink + most_expensive_snack
                        - meal_deal_cost
                ),
        );
    }
}
