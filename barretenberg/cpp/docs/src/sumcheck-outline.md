# Sumcheck Implementation
We implement a Zero-Knowledge Sumcheck protocol for relations of a very general form.  

The implementation consists of several components. 
- [Non-ZK Sumcheck:](#NonZKSumcheck) 
	We sketch an implementation of the non-zero-knowledge Sumcheck, introduce the main abstractions and the components of the proof system. In [Witness Information Leakage](#NonZKSumcheckLeakage), we determine the sources allowing the verifier to learn the witness information during Sumcheck.
	
- [Masking Round Univariates with Libra:](#LibraTechnique) 
	To prevent the witness values from leaking through the coefficients of Sumcheck round univariates, we apply a technique introduced in <a href=" https://eprint.iacr.org/2019/317">Libra: Succinct Zero-Knowledge Proofs with Optimal Prover Computation</a>. 
	Being represented in Lagrange basis, Libra masking polynomials lead to very simple formulas for contributions to Sumcheck round univariates, see [the following section](#LibraRoundUnivariates). 
	In section [Libra Costs](#LibraCosts), we assess the overhead caused by adding the Libra technique. 
	Although the contribution in field operations is almost negligible, it adds non-trivial expenses during the opening procedure.

- [Masking Evaluations of Multilinear Witnesses:](#MaskingEvalsOfWitnesses) 
	At the stage of proving their evaluations at the challenge point, the multilinear witness polynomials fed to Sumcheck must not reveal any private information. 
    We use a modification of Construction 3 described in <a href=" https://eprint.iacr.org/2019/317">Libra</a>  allowing the prover to open a new multilinear polynomial in \f$d\f$ variables, where \f$2^d\f$ is the circuit size, which is derived from the witnesses by adding a product of a random scalar and a public quadratic polynomial in \f$d\f$ variables

- [Total Costs:](#ZKCosts) The effect of adding Libra technique and masking evaluations of multilinear witnesses is assessed, and the theoretical upper bound on prover's work is compared to the implemenation costs. 

Non ZK-Sumcheck Outline {#NonZKSumcheck}
========
- - -
 ### Sumcheck Relation {#SumcheckRelation}

 Given multilinear polynomials \f$ P_1,\ldots, P_N \in \mathbb{F}[X_0,\ldots, X_{d-1}] \f$ and a polynomial \f$ F \f$ in \f$ N \f$ variables, we run Sumcheck over the polynomial
 \f{align}{
    \tilde{F}
    (X_0,\ldots, X_{d-1}) =
    pow_{\beta}(X_0,\ldots, X_{d-1}) \cdot F\left( P_1 (X_0,\ldots, X_{d-1}), \ldots, P_N (X_0,\ldots, X_{d-1}) \right)
    \f}
to establish that \f$ F(P_1(\vec \ell),\ldots, P_N(\vec \ell) ) = 0 \f$, i.e. that \f$ F \f$ is satisfied at every
point \f$\vec \ell \{0,1\}^d\f$.

 In the implementation, the relation polynomial \f$ F \f$ is specified by the Flavor.
 \todo Docs for Flavors and Relations. 

 ### Main Parameters {#MainParameters}

The following constants are used in this exposition. 

 |     Notation      |           | \f$ \sim \f$   Upper Bound |
 --------------------|---------------|-----------|
 | \f$ d \f$         | \ref multivariate_d "number of variables" in multilinear  polynomials \f$ P_1,\ldots, P_N\f$        | \f$ 20 \f$  |
 | \f$ N \f$         | number of Prover Polynomials specified by Flavor's parameter \p NUM_ALL_ENTITIES                    | \f$ 60 \f$  | 
 | \f$ N_w \f$  	 | number of Witness Polynomials specified by Flavor's parameter \p NUM_WITNESS_ENTITIES               | \f$ 17 \f$  | 
 | \f$ n \f$         | \ref multivariate_n "size of the hypercube", i.e. \f$ 2^d\f$.                                       | \f$ 2^{20} \f$ |
 | \f$ D \f$         | \ref bb::SumcheckProverRound< Flavor >::BATCHED_RELATION_PARTIAL_LENGTH "total degree of" \f$\tilde{F}\f$ as a polynomial in \f$P_1,\ldots, P_N\f$ <b> incremented by </b> 1 | \f$ 12 \f$ |
 | \f$ D_w\f$        | [maximum witness degree](#MaximumWitnessDegree) of \f$ F \f$ | \f$ 5 \f$ |

\todo Compute precise upper bounds.

#### Maximum Witness Degree {#MaximumWitnessDegree}
The significance of this parameter becomes apparent in Section [Masking Evaluations of Multilinear Witnesses](#MaskingEvalsOfWitnesses). It is formally defined as follows 
\f{align}{
	D_w = \deg_{P_1, \ldots, P_{N_w}} F(P_1,\ldots, P_{N})
\f}
where by \f$ \deg_{P_1, \ldots, P_{N_w}} \f$ we mean the <b> total degree </b> of the relation polynomial \f$ F \f$ in the witness polynomials \f$ P_1,\ldots, P_{N_w}\f$ considered as variables. 

For example, given a polynomial \f$P_1 +  P_{N_w+1} \cdot P_{N_w + 2} \cdot P_{1}^2 \cdot P_{2}\f$ in prover polynomials, where \f$N_w>2\f$, its witness degree \f$ D_w \f$ is \f$3\f$, whereas its total degree \f$D\f$ is equal to \f$ 6 \f$.  

## Sumcheck Prover Algorithm {#NonZKSumcheckProver}
- - -
We remark that prior to running Sumcheck, the prover commits to multilinear polynomials \f$P_1,\ldots, P_{N_w}\f$, and sends the commitments to the verifier and that the total sum and the relation polynomial \f$ \tilde F \f$ are public. 

The prover algorithm is implemented in the \ref bb::SumcheckProver< Flavor > "Sumcheck Prover" class. See its documentation for a more detailed description of methods described below. The Sumcheck Round routine is abstracted into \ref bb::SumcheckProverRound< Flavor > "Sumcheck Prover Round" class, which contains most computational steps.



####  Set up Prover Polynomials {#ProverPolynomialsSetup}

The polynomials \f$P_1,\ldots, P_N\f$ are abstracted in the class ProverPolynomials specific to a Flavor, e.g. \ref bb::GoblinUltraFlavor::ProverPolynomials "Goblin Ultra Flavor". 
Sumcheck Prover algorithm takes a reference to an object of this class. 

####  Compute Round Univariates and add them to Transcript {#ComputeRoundUnivariates}
The prover evaluates the round univariate 
\f{align}{
	\tilde{S}^i = \sum_{\vec \ell \in \{0,1\}^{d-1-i}} \tilde{F}\left(P_1(u_0,\ldots, u_{i-1}, X_i,\vec \ell), \ldots, P_N(u_0,\ldots, u_{i-1}, X_i,\vec \ell)\right)
\f} 
over the domain \f$ 0,\ldots, D \f$. In fact, it is more efficient to perform this computation sub-relation-wise, because the degrees of individual subrelations as polynomials in \f$ P_1,\ldots, P_N\f$ are generally smaller than \f$D\f$ defined in [Main Parameters](#MainParameters). Taking this into account, for a given subrelation of \f$F\f$, we perform expensive subrelation evaluations at points \f$(u_0,\ldots, u_{i-1}, k, \vec \ell)\f$ for \f$\ell \in \{0,1\}^{d-1-i} \f$ and \f$k\f$ from \f$0\f$ <b>only up</b> to the degree of the subrelation as a polynomial in \f$P_1,\ldots,P_N\f$ incremented by \f$1\f$. 

At the implementation level, the evaluations of \f$\tilde{S}^i\f$ are obtained using the method \ref bb::SumcheckProverRound< Flavor >::compute_univariate "compute univariate" consisting of the following sub-methods:

 - \ref bb::SumcheckProverRound::extend_edges "Extend evaluations" of linear univariate
polynomials \f$ P_j(u_0,\ldots, u_{i-1}, X_i, \vec \ell) \f$ to the domain \f$0,\ldots, D\f$. It is a cheap operation applied only once for every \f$\vec \ell \in \{0,1\}^d\f$ which allows to compute subrelations of \f$ F \f$ at such arguments. 
 - \ref bb::SumcheckProverRound::accumulate_relation_univariates "Accumulate per-relation contributions" of the extended
polynomials to auxiliary univariates \f$ T^i(X_i)\f$ defined in \ref SumcheckProverContributionsofPow "this section"
 - \ref bb::SumcheckProverRound::extend_and_batch_univariates "Extend and batch the subrelation contributions"
multiplying by the constants \f$c_i\f$ and the evaluations of \f$ ( (1−X_i) + X_i\cdot \beta_i ) \f$ stemming from \f$F\f$ being multiplied by \f$pow_{\beta}\f$.

#### Get Round Challenge {#GetRoundChallenge}

After computing Round Univariate and adding its evaluations \f$\tilde{S}^i(0),\ldots, \tilde{S}^i(D)\f$ to the transcript, the prover generates Round Challenge \f$ u_i \f$ by hashing the transcript. 

#### Populate/Update Book-keeping Table {#BookKeepingTable}
To keep prover's work linear in the number of coefficients of \f$P_1,\ldots, P_N\f$,  we \ref bb::SumcheckProver< Flavor >::partially_evaluate "populate" a table of \f$\texttt{partially_evaluated_polynomials}\f$ after getting the first challenge \f$ u_0 \f$ with the values \f$P_j(u_0,\vec \ell )\f$, namely
\f{align}{
        \texttt{partially_evaluated_polynomials}_{\ell,j} \gets  P_j(0, \ell)  + u_{0} \cdot \left(P_j(1, \vec \ell) - P_j(0, \ell)\right) \f}
for \f$ \vec \ell \in \{0,1\}^{d-1}\f$ identified with the binary representation of \f$ 0\leq \ell \leq 2^{d-1}-1\f$.

In Round \f$0< i \leq d-1\f$, the prover algorithm \ref bb::SumcheckProver< Flavor >::partially_evaluate "updates" the top \f$ 2^{d-1 - i}\f$ values in the book-keeping table 
\f{align}{
        \texttt{partially_evaluated_polynomials}_{\ell,j} \gets  \texttt{partially_evaluated_polynomials}_{2 \ell,j}  + u_{i} \cdot (\texttt{partially_evaluated_polynomials}_{2\ell+1,j} - \texttt{partially_evaluated_polynomials}_{2\ell,j}) \f} 
 where \f$\vec \ell \in \{0,1\}^{d-1-i}\f$.
 After the final update, i.e. when \f$ i = d-1 \f$, the upper row of the table contains the evaluations of Prover Polynomials at the challenge point \f$ (u_0,\ldots, u_{d-1}) \f$.
   

#### Add Claimed Evaluations to Transcript {#ClaimedEvaluations}
After computing the last challenge \f$ u_{d-1} \f$ in Round \f$ d-1 \f$ and updating \f$
\texttt{partially_evaluated_polynomials} \f$, the prover looks into the top row of the table containing evaluations
\f$P_1(u_0,\ldots, u_{d-1}), \ldots, P_N(u_0,\ldots, u_{d-1})\f$ and concatenates these values with the last challenge
to the transcript.




## Sumcheck Verifier Algorithm {#NonZKSumcheckVerifier}
- - -

The verifier algorithm is implemented in the \ref bb::SumcheckVerifier< Flavor > "Sumcheck Verifier" class. See its documentation for a more detailed description of methods described below. The Sumcheck Round verification routine is abstracted into \ref bb::SumcheckVerifierRound< Flavor > "Sumcheck Verifier Round" class.

The verifier's work reduces to the following. 

For \f$ i = 0,\ldots, d-1\f$:
  -  Using \ref bb::BaseTranscript::receive_from_prover "receive_from_prover" method from \ref bb::BaseTranscript< TranscriptParams > "Base Transcript Class", extract the evaluations of Round Univariate \f$ \tilde{S}^i(0),\ldots, \tilde{S}^i(D) \f$ from the transcript.
  - \ref bb::SumcheckVerifierRound< Flavor >::check_sum "Check target sum": \f$\quad \sigma_{
 i } \stackrel{?}{=}  \tilde{S}^i(0) + \tilde{S}^i(1)  \f$.
  - \ref bb::BaseTranscript::get_challenge "Get the next challenge"  \f$u_i\f$ by hashing the transcript.
 method.
  - \ref bb::SumcheckVerifierRound< Flavor >::compute_next_target_sum "Compute next target sum" :\f$ \quad \sigma_{i+1}
 \gets \tilde{S}^i(u_i) \f$

### Verifier's Data before Final Step {#SumcheckVerifierData}
Entering the final round, the verifier has already checked that 
\f$\quad \sigma_{ d-1 } =  \tilde{S}^{d-1}(0) + \tilde{S}^{d-1}(1)  \f$
and computed \f$\sigma_d = \tilde{S}^{d-1}(u_{d-1})\f$.

### Final Verification Step {#NonZKSumcheckVerification}
- Extract claimed evaluations of prover polynomials \f$P_1,\ldots, P_N\f$ at the challenge point \f$
 (u_0,\ldots,u_{d-1}) \f$ from the transcript and \ref bb::SumcheckVerifierRound< Flavor >::compute_full_honk_relation_purported_value "compute evaluation:"
 \f{align}{\tilde{F}\left( P_1(u_0,\ldots, u_{d-1}), \ldots, P_N(u_0,\ldots, u_{d-1}) \right)\f}

- Compare \f$ \sigma_d \f$ against the evaluation of \f$ \tilde{F} \f$ at \f$P_1(u_0,\ldots, u_{d-1}), \ldots,
 P_N(u_0,\ldots, u_{d-1})\f$:
  \f{align}{\quad  \sigma_{ d } \stackrel{?}{=} \tilde{F}\left(P_1(u_{0}, \ldots, u_{d-1}),\ldots, P_N(u_0,\ldots,
 u_{d-1})\right)\f}


## Witness Information Leakage {#NonZKSumcheckLeakage}

--------

As explained in Section 13.3 of <a href="https://people.cs.georgetown.edu/jthaler/ProofsArgsAndZK.html">Proofs, Arguments, and Zero-Knowledge</a>, there are two main sources that leak prover's private information: 
- Evaluations of Round Univariates \f$ \tilde{S}^i\f$
- Evaluations of witness polynomials \f$P_1,\ldots, P_{N_w}\f$ that the prover sends and proves at the last step of Sumcheck.

These issues are resolved by enhancing Sumcheck with a technique that randomizes any given number of evaluations of \f$\tilde{S}^{i} \f$ and a technique that randomizes evaluations of witness polynomials \f$ P_1,\ldots, P_{N_w} \f$ at the challenge point \f$(u_0,\ldots, u_{d-1})\f$ obtained in Sumcheck.

-------

Masking Round Univariates with Libra {#LibraTechnique}
========
-----

## Main Idea of Libra {#LibraMainIdea}

To prevent the witness information leakage through the Round Univariates determined by their evaluations over the domain  \f$ \{0,\ldots, \tilde{D}\}\f$, where \f$\tilde{D} \geq D\f$, the Sumcheck Prover masks them using a <b>low-degree</b> multivariate polynomial
\f{align}{
	G \gets \sum_{i=0}^{d-1} g_{i}(X_i),
\f}
where
\f{align}{
	g_{i} = \sum_{j=0}^{\tilde{D}} g_{i,j} \cdot L_{j,\{0,\ldots, D\}}(X_i) \quad  \text{for } (g_{i,j}) \gets_{\$} \mathbb{F}^{d\cdot (\tilde{D}+1)}
\f}
and \f$L_{j, \{0,\ldots, \tilde{D}\}}\f$ is the \f$j\f$th univariate Lagrange polynomial for the domain \f$\{0,\ldots, \tilde{D}\}\f$. Recall that \f$\deg_{X_i} \left(L_{j, \{0,\ldots, \tilde{D}\}} (X_i)\right) = \tilde{D}\f$.

Set 
\f{align}{
	\gamma \gets \sum_{\vec \ell \in \{0,1\}^{d}} G(\vec \ell)
\f}
as the value that the honest prover sends to the verifier, and let \f$\rho\f$ be the verifier's challenge. 
Then instead of proving that \f$\sum \tilde{F}\left(\vec \ell\right) =\sigma\f$ as in  [Non-ZK Sumcheck](\ref NonZKSumcheck), we run the protocol that establishes that 
\f{align}{
	\sum_{\vec \ell \in\{0,1\}^{d}} \left(\tilde{F}(P_1(\vec \ell), \ldots, P_N(\vec \ell)) + \rho \cdot G(\vec \ell)\right) = \sigma + \rho \cdot \gamma. 
\f}
### Properties of Libra Masking Polynomial {#PropertiesOfTheMaskingPolynomial}

Observe that \f$ G \f$ has several important properties
- For \f$ i = 0,\ldots, d-1\f$, the partial degrees \f$ \deg_{X_i} G = \tilde{D}\f$.
- The coefficients of \f$ G \f$ are independent and uniformly distributed. 
- Evaluations of \f$ G \f$ at \f$ \vec \ell \in \{0,1\}^d\f$ and related Sumcheck Round Univariates are efficiently computable.

The first two properties imply that the evaluations over the domain \f$ \{0,\ldots, \tilde{D}\}\f$ defining <b>Libra Round Univariates </b>, i.e.  round univariates for \f$ G \f$,  are independent and uniformly distributed. 
Moreover, since Round Univariates for \f$ \tilde{F} + \rho\cdot G\f$ are the sums of respective unvariates, the second property and the condition \f$ \tilde{D}\geq D \f$ ensure that the evaluations \f$ \tilde{S}^i(0),\ldots,\tilde{S}^i(\tilde D)\f$ defined in [Compute Round Univariates](#ComputeRoundUnivariates) are hidden by random scalars obtained as evaluations of Libra Round Univariates, which are described explicitly [below](#LibraRoundUnivariates).

### Example {#LibraPolynomialExample}
If in every round of Sumcheck, the prover aims to hide only \f$2\f$ evaluations the Round Univariate, i.e. if \f$\tilde{D} = 1\f$, the masking polynomial \f$ G \f$ has the following form 
\f{align}{
	G = \left( g_{0,0} (1- X_0) + g_{0,1} X_0 \right) + \ldots +  \left( g_{d-1,0} (1- X_{d-1}) + g_{d-1,1} X_{d-1} \right).
\f}  
## Implementation {#LibraImplementation}

### Committing to Libra Masking Polynomial {#LibraCommitments}

To commit to multivariate polynomial \f$ G \f$, the prover commits to the tuple of univariate polynomials \f$ (g_0,\ldots, g_{d-1})\f$.

### Computing Target Sum {#LibraTargetSum} 
Since \f$G\f$ is a polynomial of a very special form, the computation of \f$\gamma\f$ reduces to the following
\f{align}{
	\sum_{\vec \ell \in \{0,1\}^{d}} G(\vec \ell) = \sum_{i=0}^{d-1} \sum_{\vec \ell \in \{0,1\}^{d}} g_{i}(\ell_i) = 2^{d-1} \sum_{i = 0}^{d-1} \left( g_i(0) + g_i(1) \right),
\f}
since the evaluations of \f$ g_i \f$ at \f$\vec \ell \in \{0,1\}^{d}\f$ depend only on \f$ \ell_i \f$, and therefore, there are \f$2^{d-1}\f$ summands \f$ g_i(0) \f$ corresponding to the points \f$\vec \ell\f$ with \f$\ell_i=0\f$ and \f$2^{d-1}\f$ summands \f$ g_i(1) \f$ corresponding to \f$\vec \ell\f$ with \f$\ell_i=1\f$. 

We set
\f{align}{
	\texttt{libra_total_sum} \gets 2^{d-1} \sum_{i = 0}^{d-1} \left( g_i(0) + g_i(1) \right)
\f}

### Pre-computed Data and Book-Keeping {#LibraBookKeeping}
As in [Sumcheck Book-keeping](#BookKeepingTable), we use a table of evaluations of Libra univariates to avoid extra computational costs.
Namely, before Round \f$ i \f$, the prover needs the table of values
\f{align}{
	\texttt{libra_table}_{j,k} \gets \rho \cdot 2^{d-1-i} \cdot g_{j,k} \text{ for } j= i,\ldots, d-1, \text{ and } k=0,\ldots, \tilde{D}
\f} 
and the term 
\f{align}{
	\texttt{libra_running_sum} \gets \rho \cdot 2^{d-1-i}\left( \sum_{j=0}^{i-1}g_j(u_j) + \sum_{j = i+1}^{d-1} ( g_{j,0} + g_{j,1}) \right).
\f}

### First Round {#LibraFirstRound}

The prover computes first Libra round univariate
\f{align}{
	\texttt{libra_univariate}_0(X_0) = \rho \cdot  \sum_{\vec \ell \in \{0,1\}^{d-1}} G(X_0,\vec \ell) =
	 2^{d-1} \rho\cdot g_0(X_0) + 2^{d-1} \rho \cdot \sum_{i=1}^{d-1}\left(g_i(0)+g_i(1)\right)
\f}
which could be expressed as follows
\f{align}{
	\texttt{libra_univariate}_0 (k) \gets  \texttt{libra_table}_{0,k} + \texttt{libra_running_sum}
\f}
for \f$k=0,\ldots, \tilde{D}\f$.

When the prover receives the challenge \f$u_0\f$, it computes the value \f$g_0(u_0)\f$ using \ref bb::Univariate::evaluate "evaluate" method, updates the running sum 
\f{align}{
	\texttt{libra_running_sum} \gets 2^{-1} \cdot \left( (g_0(u_0) + \texttt{libra_running_sum}) - (\texttt{libra_table}_{1,0} + \texttt{libra_table}_{1,1})\right)
\f}
and updates the libra table by releasing the first column and multiplying reamining terms by \f$1/2\f$.

### Round Univariates in Subsequent Rounds {#LibraRoundUnivariates}
Similarly, to compute the contribution of Libra masking polynomial \f$G\f$ to the round univariates \f$\tilde{S}_i\f$ defined in [Compute Round Univariates](#ComputeRoundUnivariates), consider  
\f{align}{
	\texttt{libra_univariate}_i(X_i) =  \rho \cdot \sum_{\vec \ell \in \{0,1\}^{d-1 - i}} G(u_0,\ldots, u_{i-1}, X_{i}, \vec \ell) = 
	\rho \cdot 2^{d-1 - i}  \left( \sum_{j = 0}^{i-1} g_j(u_{j}) + g_{i}(X_i) + \sum_{j=i+1}^{d-1} \left(g_{j,0} + g_{j,1}\right) \right)
\f}  
Therefore, the contribution of the \f$\texttt{libra_univariate}_{i}(X_{i})\f$ at \f$X_{i} = k\f$ to \f$\tilde{S}^i(k)\f$, where \f$k=0,\ldots, \tilde{D}\f$, is given by the formula 
\f{align}{
	\texttt{libra_univariate}_i(k) = \rho \cdot 2^{d-1-i} \left(\sum_{j = 0}^{i-1} g_j(u_{j}) + g_{i,k}+ \sum_{j=i+1}^{d-1}\left(g_{j,0}+g_{j,1}\right)\right) =  \texttt{libra_table}_{i,k} + \texttt{libra_running_sum}.
\f}

### Updating Partial Evaluations {#LibraUpdatePartialEvaluations} 
In Rounds \f$ i = 1,\ldots d-2\f$, after correcting Sumcheck round univariate \f$S_{i}(X_{i})\f$ by \f$ \texttt{libra_univariate}_i(X_i)\f$, the prover gets the challenge \f$u_{i}\f$, computes the value \f$\texttt{libra_univariate}_{i}(u_{i})\f$ and updates the running sum 
\f{align}{
	\texttt{libra_running_sum} \gets 2^{-1} \cdot \left( (g_i(u_i) + \texttt{libra_running_sum}) - (\texttt{libra_table}_{i+1,0} + \texttt{libra_table}_{i+1,1})\right)
\f}



### Final Round {#LibraFinalRound}
After sending the evaluations of \f$\texttt{libra_univariate}_{d-1}\f$ at over the domain \f$\{0,\ldots, \tilde{D}\}\f$, the prover gets the last challenge \f$u_{d-1}\f$ and has to send the claimed evaluation  \f$G(u_0,\ldots, u_{d-1})\f$. It boils down to sending and proving the evaluations 
\f{align}{
	v_i = g_i(u_i) \text{ for } i = 0,\ldots, d-1.
\f}
## Libra Costs {#LibraCosts}

The overhead in prover's field operations is linear in \f$ d\cdot \tilde D \f$ with a small constant and therefore, is negligible compared to the number of field operations performed during the main Sumcheck routine.

The main expenses are caused by proving the evaluations of \f$ g_i (u_i) = v_i\f$ in the [Final Round](\ref LibraFinalRound). 
Using the PCS introduced in Section 4 of <a href=" https://eprint.iacr.org/2020/081">Efficient polynomial commitment schemes for multiple points and polynomials</a> also known as Shplonk, we reduce the costs of opening \f$ d \f$ univariate polynomials \f$ g_i \f$, each at different \f$ u_i \f$ to the following:

<table>
<tr><th>       <th>Prover       <th>Verifier
<tr>		<td> <b> Group Operations </b> </td>
			<td> \f$ 2 \cdot \tilde D+1\f$ </td>
			<td> \f$ d + 3 \f$ </td>
		</tr>
		<tr>
			<td> <b> Field Operations </b> </td>
			<td>\f$ O\left(d\cdot (\tilde{D} +1) + \tilde{D} \log(\tilde{D})\right)\f$ </td>
			<td></td>
		</tr>
		<tr>
			<td> <b> Pairings </b> </td>
			<td></td>
			<td> 2 </td>
		</tr>
		<tr>
			<td> <b> Proof Size </b> </td>
			<td colspan="2"> 2 group elements </td>
		</tr>
</table>

Masking Evaluations of Multilinear Witnesses {#MaskingEvalsOfWitnesses}
========
- - -

At the last step of Sumcheck, the Prover adds the evaluations of multilinear polynomials \f$P_1,\ldots, P_N \f$ at the challenge point \f$\vec u = (u_0,\ldots, u_{d-1})\f$ to the trasncript. 

Let \f$ N_w\leq N\f$ and assume that \f$ P_1,\ldots, P_{N_w}\f$ are witness polynomials. 
To mask their evaluations at the challenge point\f$\vec u\f$, the prover samples 
\f{align}{\rho_1,\ldots \rho_{N_w} \gets_{\$} \mathbb{F}^{N_w}\f} 
and sets 
\f{align}{
	\texttt{masked_witness_polynomial}_j(X_0,\ldots, X_{d-1}) = \widehat{P}_j \gets P_j(X_0,\ldots, X_{d-1}) + \rho_j \cdot \left(\sum_{k=0}^{d-1} X_k(1-X_k) \right).
\f}


Note that for any relation \f$F\f$, we have
\f{align}{
	\sum_{\ell \in \{0,1\}^d} pow_{\beta}(X_0,\ldots, X_{d-1}) \cdot F\left(P_1(\ell), \ldots, P_N(\ell)\right) 
	= pow_{\beta}(X_0,\ldots, X_{d-1}) \sum_{\ell \in \{0,1\}^d} F\left(\widehat{P}_1(\ell), \ldots, \widehat{P}_{N_w}(\ell), P_{N_w+1}(\ell), \ldots, P_{N}(\ell)\right)
\f}
as \f$P_j\f$ and \f$\widehat{P}_j\f$ agree on the hypercube \f$\{0,1\}^d\f$ for \f$j=1,\ldots, N_w\f$.

### Committing to Prover Polynomials {#CommittingToMaskedWitnesses}

The prover commits to \f$P_j\f$ for \f$j=1,\ldots, N\f$ and to \f$\rho_j\f$ for \f$ j=1, \ldots, N_w\f$ as multilinear polynomials and sends the commitments to the verifier. 

### Evaluation Phase {#MaskedEvaluationProtocol}
At the end of Sumcheck, instead of proving evaluations of witness polynomials \f$P_j\f$ at \f$\vec u\f$ for \f$j=1,\ldots, N_w\f$, the prover opens multilinear polynomials
\f{align}{
	\widehat{P}_j^{\vec u} \gets P_j(X_0,\ldots, X_{d-1}) + \rho_j \cdot \left(\sum_{k=0}^{d-1} u_k(1-u_k) \right).
\f}
It is important to notice that the verifier could evaluate public polynomial \f$\sum_{k=0}^{d-1} X_k(1-X_k)\f$ and derive the commitments to \f$\widehat{P}_j^{\vec u}\f$ on its own.

The remaining prover polynomials \f$P_{N_w+1}, \ldots, P_{N}\f$ are evaluated at \f$ \vec u \f$ and their evaluations are proved without any modifications.

### Security Check {#SecurityCheck}
Before proving the evaluations of \f$\widehat{P}_j^{\vec u} \f$, the prover checks that \f$\vec u\f$ does not satisfy the equation \f$\sum_{k=0}^{d-1} X_k(1-X_k)  = 0\f$, which generally has many solutions outside of the hypercube \f$\{0,1\}^d\f$.

### Degrees of Round Univariates  {#DegreesRoundUnivariatesZK}

Since masked witness polynomials \f$\widehat{P}_j\f$ are of degree \f$2\f$ in every variable, the degree of Sumcheck round univariates is also affected.
Namely, we set
\f{align}{
	\tilde{D} = \max_{i\in\{0,\ldots,d-1\}} \left\{\deg_{X_i} F\left(\widehat{P}_1(X_0,\ldots,X_{d-1}),\ldots, \widehat{P}_{N_w}(X_0,\ldots, X_{d-1}),\widehat{P}_{N_w+1}(X_0,\ldots, X_{d-1}), \ldots, \widehat{P}_{N}(X_0,\ldots, X_{d-1}) \right) \right\} \leq D + D_{w}
\f}
for \f$D\f$ and \f$ D_w \f$  defined in [Parameters](\ref MainParameters). 

In every round of Sumcheck, the Prover sends univariate polynomials \f$\tilde{S}^i(X_i)\f$ represented by their evaluations at \f$X_i = 0,\ldots, \tilde{D}\f$. 

Note that \f$ \tilde{D} \f$ <b> sets up </b> the corresponding parameter of [Libra](#LibraImplementation)

### Book-keeping Tables {#BookKeepingMaskingWitnesses}
To reduce the computation costs, the prover precomputes the table 
\f{align}{
    \texttt{masking_terms_evaluations}_{k,j}\gets \rho_j \cdot (1-k) k 
\f}
for \f$j=1, \ldots, N_w\f$ and \f$ k=2,\ldots, \tilde{D} \f$ and stores the vector of running quadratic terms 
\f{align}{
   \texttt{running_quadratic_term}_j \gets  \rho_j \cdot \sum_{k=0}^{i-1}  (1-u_k) u_k.
\f} 


### Computing Evaluations of Round Univariates {#RoundUnivariatesMaskedEval}
In Round \f$i \in \{0,\ldots, d-1\}\f$, the prover computes univariate polynomials
\f{align}{
	\widehat{S}^i(X_i) = \sum_{\vec\ell \in \{0,1\}^{d-1-i}} F\left(\widehat{P}_1(u_0,\ldots, u_{i-1}, X_i, \vec \ell),\ldots,\widehat{P}_{N_w}(u_0,\ldots, u_{i-1}, X_i, \vec \ell), P_{N_w+1}(u_0,\ldots, u_{i-1}, X_i, \vec \ell), \ldots, P_{N}(u_0,\ldots, u_{i-1}, X_i, \vec \ell) \right)
\f}
which reduces to computing at most  \f$ (D+ D_w + 1) \times N \times 2^{d-1 - i}\f$ values
\f{align}{
	&\ P_j(u_0,\ldots, u_{i-1}, k, \vec \ell) + \rho_j \cdot \sum_{k=0}^{i-1} u_k(1-u_k) + \rho_j\cdot (1-k) k \quad  \text{ for } j=1,\ldots, N_w\\
    &\ P_j(u_0,\ldots, u_{i-1}, k, \vec \ell) \quad \text { for } j= N_w+1,\ldots, N
\f}
The values \f$ \texttt{running_quadratic_term}_j = \rho_j \cdot \sum_{k=0}^{i-1} u_k(1-u_k)\f$ are available from Round \f$i-1\f$. 
The products \f$ \rho_j \cdot (1-k) k\f$ are taken from the table \f$ \texttt{masking_terms_evaluations}\f$. 

The prover performs an extra addition per evaluation \f$\widehat{P}_j(u_0,\ldots, u_{i-1}, k, \vec \ell)\f$ for \f$k=0,1\f$ and two extra additions per evaluation for \f$k=2,\ldots, D+D_w\f$ compared to evaluating the original witness polynomials \f$P_j\f$. 
It results in \f$2 (D+D_w) N_w (2^d-1) \f$ extra additions compared to [Non-ZK-Sumcheck](#NonZKSumcheck). 

Upon receiving the round challenge \f$ u_i\f$, the prover prepares the correcting term for the next round
\f{align}{
    \texttt{running_quadratic_terms}_j \gets  \texttt{running_quadratic_terms}_j + \rho_j \cdot (1-u_i) u_i .
\f}

### Witness Evaluation Masking Costs {#MaskingCosts}
In contrast to non-ZK-Sumcheck, the prover needs to compute \f$\tilde{D} \sim D+D_w \f$ evaluations of round univariates \f$S_i\f$, which results in
\f{align}{
	((D+D_w)\cdot N +  C_a \cdot (D+D_w) + 2\cdot N + 2\cdot (D+D_w) N_w ) (2^d - 1)
\f}
addditions and
\f{align}{
	(C_m\cdot (D+D_w) + N) \cdot (2^d -1)
\f}
multiplications, where \f$C_a\f$ and \f$C_m\f$ are constants corresponding to the number of additions and multiplications required to evaluate the relation polynomial \f$F\f$.

The overhead is summarized in the following table. 

<table>
<tr><th>       <th>Prover       <th>Verifier
<tr>		<td> <b> Group Operations </b> </td>
			<td> \f$ + N_w\f$ MSMs of size \f$2^d\f$ (same group element) </td>
			<td> \f$ + N_w\f$ MSMs of size 2 </td>
		</tr>
		<tr>
			<td> <b> Field Operations </b> </td>
			<td>\f$\times(1+ D_w/D) \f$ </td>
			<td>\f$\times(1+D_w/D) \f$</td>
		</tr>
		<tr>
			<td> <b> Communication </b> </td>
			<td colspan="2"> \f$ + N_w\f$ group elements; \f$ +D_w\cdot d\f$ field elements  </td>
		</tr>
</table>

ZK Costs {#ZKCosts}
========
- - -

The total costs of ZK Sumcheck are obtained from [Libra Costs](#LibraCosts) and  [Witness Evaluation Masking Costs](#MaskingCosts).

<table>
<tr><th>       <th>Prover       <th>Verifier
<tr>		<td> <b> Group Operations </b> </td>
			<td><p> \f$+ d\f$ MSMs of size \f$(D+D_w)\f$ = [Libra Commitments](#LibraCommitments) </p>
			    <p> \f$+ \left(2 \cdot (D+D_w) D+1\right)\f$ group ops = <b>shplonk</b> </p>
			    <p> \f$+ N_w\f$  MSMs of size \f$2^d\f$ (same group element multiplied by \f$\rho_1,\ldots,\rho_{N_w}\f$) </p> 
			</td>
			<td><p> \f$ + (d + 3) \f$ group ops </p>
				<p> \f$ + N_w\f$ MSMs of size \f$2\f$</p>  
			</td>
		</tr>
<tr>
			<td> <b> Field Operations </b> </td>
			<td> \f$ \times D_w/D \f$ </td>
			<td> \f$ \times D_w/D \f$ </td>
</tr>
<tr>
			<td> <b> Pairings </b> </td>
			<td></td>
			<td> + 2 </td>
</tr>
<tr>
			<td> <b> Communication </b> </td>
			<td colspan="2"> <p> \f$+ (d+ 2)\f$ group elements for Libra;  \f$+N_w\f$ group elements for witness masking</p> 
							 <p> \f$+ D_w \cdot d \f$ field elements </p></td>
</tr>
</table>

## Theoretic Field Operations vs. Implementation

The table above sets a reasonable upper bound on the amount of prover's field operations. 
However, in the implementation, the relation \f$ F \f$ is computed as a vector of its subrelations, which allows us to decrease the costs of computing the round univariates. Namely, for a given subrelation \f$ F_j \f$, its maximum partial degree \f$D_j\f$ and  its witness degree \f$D_{w,j} \f$ are generally less than \f$ D\f$ and  \f$ D_w \f$, respectively. 
Therefore, we compute \f$ F_j \f$'s contribution to Sumcheck Round Univariates by evaluating the univariate polynomial
\f{align}{
	\sum_{\vec \ell\in \{0,1\}^{d-1-i}} pow_{\beta}(u_0,\ldots, u_{i-1}, X_i, \vec \ell) \cdot F_j(u_0,\ldots, u_{i-1}, X_i,\vec \ell)
\f}
at \f$ X_i = 0,\ldots, D_i + D_{w,i}\f$ and extend the resulting univariate of degree \f$D_j+D_{w,j}\f$ to the entire domain  \f$\{ 0,\ldots, D+D_w\}\f$, which is way cheaper than evaluating the sum above at \f$ X_i = D_{j}+ D_{w,j}+1, \ldots, D+ D_w \f$

