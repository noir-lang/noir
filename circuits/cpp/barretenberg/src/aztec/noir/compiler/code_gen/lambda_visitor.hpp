template <typename ReturnType, typename... Lambdas>
struct lambda_visitor : public boost::static_visitor<ReturnType>, public Lambdas... {
    lambda_visitor(Lambdas... lambdas)
        : Lambdas(lambdas)...
    {}
};

template <typename ReturnType, typename... Lambdas>
lambda_visitor<ReturnType, Lambdas...> make_lambda_visitor(Lambdas... lambdas)
{
    return { lambdas... };
}
