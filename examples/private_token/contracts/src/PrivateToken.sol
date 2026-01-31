// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title UltraVerifier
 * @dev Interface for the Noir-generated verifier contract
 * This will be replaced by the actual generated verifier
 */
interface IUltraVerifier {
    function verify(bytes calldata proof, bytes32[] calldata publicInputs) external view returns (bool);
}

/**
 * @title PrivateToken
 * @dev Privacy-preserving token with hidden balances and addresses
 * Uses Zero-Knowledge Proofs (Noir) to verify transactions
 */
contract PrivateToken {
    // Verifier contracts for different proof types
    IUltraVerifier public transferVerifier;
    IUltraVerifier public mintVerifier;
    
    // Owner for admin functions
    address public owner;
    
    // Track used commitments and nullifiers
    mapping(bytes32 => bool) public commitments;
    mapping(bytes32 => bool) public nullifiers;
    
    // Commitment merkle tree root (for more advanced implementations)
    bytes32 public commitmentRoot;
    uint256 public commitmentCount;
    
    // Events
    event CommitmentAdded(bytes32 indexed commitment, uint256 indexed index);
    event NullifierUsed(bytes32 indexed nullifier);
    event PrivateTransfer(
        bytes32 indexed nullifier, 
        bytes32 senderOutput, 
        bytes32 recipientOutput,
        uint256 timestamp
    );
    event PrivateMint(bytes32 indexed commitment, uint256 requestId, uint256 timestamp);
    event VerifierUpdated(string verifierType, address newVerifier);
    
    // Errors
    error CommitmentAlreadyExists();
    error NullifierAlreadyUsed();
    error InvalidProof();
    error CommitmentNotFound();
    error OnlyOwner();
    error ZeroAddress();
    
    modifier onlyOwner() {
        if (msg.sender != owner) revert OnlyOwner();
        _;
    }
    
    constructor(address _transferVerifier, address _mintVerifier) {
        owner = msg.sender;
        transferVerifier = IUltraVerifier(_transferVerifier);
        mintVerifier = IUltraVerifier(_mintVerifier);
    }
    
    /**
     * @dev Update verifier contracts (admin only)
     */
    function setTransferVerifier(address _verifier) external onlyOwner {
        if (_verifier == address(0)) revert ZeroAddress();
        transferVerifier = IUltraVerifier(_verifier);
        emit VerifierUpdated("transfer", _verifier);
    }
    
    function setMintVerifier(address _verifier) external onlyOwner {
        if (_verifier == address(0)) revert ZeroAddress();
        mintVerifier = IUltraVerifier(_verifier);
        emit VerifierUpdated("mint", _verifier);
    }
    
    /**
     * @dev Mint tokens privately
     * @param proof ZK proof of valid minting
     * @param publicInputs [output_commitment, mint_request_id]
     */
    function mint(bytes calldata proof, bytes32[] calldata publicInputs) external {
        require(publicInputs.length == 2, "Invalid public inputs");
        
        bytes32 outputCommitment = publicInputs[0];
        
        // Verify commitment doesn't exist
        if (commitments[outputCommitment]) revert CommitmentAlreadyExists();
        
        // Verify the proof
        if (!mintVerifier.verify(proof, publicInputs)) revert InvalidProof();
        
        // Store commitment
        commitments[outputCommitment] = true;
        commitmentCount++;
        
        emit CommitmentAdded(outputCommitment, commitmentCount);
        emit PrivateMint(outputCommitment, uint256(publicInputs[1]), block.timestamp);
    }
    
    /**
     * @dev Transfer tokens privately
     * @param proof ZK proof of valid transfer
     * @param publicInputs [input_commitment, output_commitment_sender, 
     *                      output_commitment_recipient, nullifier, new_nonce]
     */
    function transfer(bytes calldata proof, bytes32[] calldata publicInputs) external {
        require(publicInputs.length == 5, "Invalid public inputs");
        
        bytes32 inputCommitment = publicInputs[0];
        bytes32 outputCommitmentSender = publicInputs[1];
        bytes32 outputCommitmentRecipient = publicInputs[2];
        bytes32 nullifier = publicInputs[3];
        
        // Verify input commitment exists
        if (!commitments[inputCommitment]) revert CommitmentNotFound();
        
        // Verify nullifier hasn't been used (prevent double-spend)
        if (nullifiers[nullifier]) revert NullifierAlreadyUsed();
        
        // Verify output commitments don't exist
        if (commitments[outputCommitmentSender]) revert CommitmentAlreadyExists();
        if (commitments[outputCommitmentRecipient]) revert CommitmentAlreadyExists();
        
        // Verify the proof
        if (!transferVerifier.verify(proof, publicInputs)) revert InvalidProof();
        
        // Update state
        nullifiers[nullifier] = true;
        commitments[outputCommitmentSender] = true;
        commitments[outputCommitmentRecipient] = true;
        commitmentCount += 2;
        
        emit NullifierUsed(nullifier);
        emit CommitmentAdded(outputCommitmentSender, commitmentCount - 1);
        emit CommitmentAdded(outputCommitmentRecipient, commitmentCount);
        emit PrivateTransfer(nullifier, outputCommitmentSender, outputCommitmentRecipient, block.timestamp);
    }
    
    /**
     * @dev Check if a commitment exists
     */
    function hasCommitment(bytes32 commitment) external view returns (bool) {
        return commitments[commitment];
    }
    
    /**
     * @dev Check if a nullifier has been used
     */
    function isNullifierUsed(bytes32 nullifier) external view returns (bool) {
        return nullifiers[nullifier];
    }
    
    /**
     * @dev Get total number of commitments
     */
    function getCommitmentCount() external view returns (uint256) {
        return commitmentCount;
    }
}
