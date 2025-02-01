# Privacy Policy

## Overview
This Privacy Policy explains how we collect, use, and protect data that you voluntarily share through our software for the purpose of building an open-source dataset for AI training through imitation learning.

## Data Collection
Our service operates without user accounts or personal identification. When you choose to record a demonstration using our software, we collect:
- Screen recordings
- Mouse movements
- Keyboard actions (including typed text)
- Window layout metadata
- Accessibility tree data
- Cryptocurrency wallet public key (only if connected for receiving rewards)

The metadata we collect (window layout and accessibility trees) is used solely for AI training purposes.

Please note that since we record keyboard actions, any personal information you type during a recording session may be captured. We provide tools to help you review and remove such information before submission.

We do NOT intentionally collect:
- System-level personal information
- Account credentials
- Cryptocurrency wallet private keys or other sensitive wallet data
- Hardware IDs (HWID)
- Computer names
- System identifiers
- Any other machine-identifying metadata

You maintain full control over your recordings:
- You can review all recordings before submission
- You can use our built-in tools to detect and remove personally identifiable information (PII)
- You can block specific applications or windows from being recorded
- You can delete recordings entirely before submission

## Data Usage
By uploading your recordings, you acknowledge and agree that:
1. Your submitted recordings will be used to train artificial intelligence models
2. Your data will be used in two ways:

   Internal Usage:
   - Stored in our internal datasets with associated wallet public keys
   - Used to train AI models
   - Never shared publicly in raw form
   - Model weights trained on this data may be released publicly
   
   Public Datasets:
   - Processed to attempt to remove personally identifiable information (PII)
   - Wallet public keys removed for anonymization
   - Released under the MIT License
   - Published on Hugging Face
   - Available for commercial and non-commercial use

3. These public datasets and model weights may be used by researchers, organizations, and other third parties for any purpose allowed under the MIT License
4. While we process public datasets to remove PII, you should use the provided tools during recording to ensure sensitive information is not captured

Note: We maintain the original recordings with associated wallet public keys on our servers to enable user deletion rights and reward distribution.

## Data Protection
We implement several measures to protect your privacy:

### Blocking and Filtering Tools
- Application and Window Blocking:
  - Block specific applications or windows from recording
  - Any data recorded while a blocked window is active is automatically excluded
  - This includes all associated keyboard input, mouse movements, and metadata during those time periods
- AI-Powered Semantic Filtering:
  - Specify topics or keywords you want to block
  - Our AI automatically detects and censors any recording segments that are semantically related to your blocked topics
  - This helps prevent accidental capture of sensitive content even if expressed in different words

### Review and Control
- Built-in PII detection and removal tools to help identify and remove personal information
- Pre-submission review capabilities
- Secure data transmission and storage

While we provide tools to help remove personal information, please be mindful that any text you type or information you display during recording may be captured. We encourage you to:
- Use our PII detection tools
- Configure topic/keyword blocking before starting a recording
- Block applications or windows that might contain sensitive information
- Carefully review recordings before submission
- Avoid typing or displaying sensitive information during recordings

## Compensation
Users who contribute recordings may be eligible for compensation according to our current payment terms and conditions.

## Data Deletion Rights and Limitations

### Deletion Options
1. Through Your Wallet (Recommended):
   - Use the deletion button in your history page
   - Requires connecting the original wallet used for submission
   - Immediate automated processing

2. Through Email Support:
   - If you've lost access to your private key
   - Email privacy@viralmind.ai
   - Include your wallet public address in the request
   - We will process the deletion of recordings associated with that wallet

In both cases, we will:
- Remove your data from our servers
- Exclude your data from future public dataset releases
- Unlink your wallet public key from stored recordings

### Important Limitations
You acknowledge and understand that:

1. Verification Requirements:
   - All deletion requests must include your wallet's public address
   - This is the only way we can identify your specific recordings
   - We cannot process deletion requests without a wallet address
   
2. Technical Limitations:
   - Previously released public datasets containing your data will continue to exist
   - We cannot recall or delete data from already-released public datasets
   - Model weights trained on your data before deletion will remain in use
   
3. Recommendations:
   - Securely back up your wallet if you want to maintain direct deletion rights
   - Save your wallet's public address in case you need to request deletion via email
   - Consider these limitations before submitting recordings
   - Use the provided privacy tools during recording to prevent capturing sensitive information

By using our service, you understand and accept these technical limitations regarding data deletion.

## Updates to Privacy Policy
We reserve the right to update this privacy policy as needed. Users will be notified of any significant changes.