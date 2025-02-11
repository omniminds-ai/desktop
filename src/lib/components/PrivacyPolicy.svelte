<script lang="ts">
import RecordingBadge from './RecordingBadge.svelte';
</script>

<div class="bg-white rounded-lg shadow-lg overflow-hidden">
  <div class="p-6 space-y-6">
    <div class="flex items-center justify-between">
      <h2 class="text-2xl font-bold">Privacy Policy</h2>
    </div>

    <div class="h-[400px] overflow-y-auto pr-4 space-y-6 text-gray-700">
      <section>
        <h3 class="text-lg font-semibold mb-2">Overview</h3>
        <p>This Privacy Policy explains how we collect, use, and protect data that you voluntarily share through our software for the purpose of building an open-source dataset for AI training through imitation learning.</p>
      </section>

      <section>
        <h3 class="text-lg font-semibold mb-2">Data Collection</h3>
        <p>Our service operates without user accounts or personal identification.</p>
        
        <div class="flex items-center flex-col gap-2 my-4">
          <p>Only when you see</p>
          <div class="bg-black/80 rounded-lg border border-purple-500/50 p-2 text-white shadow-lg backdrop-blur-sm">
            <RecordingBadge state="recording" time={0} />
          </div>
          <p>do we collect the following data:</p>
        </div>

        <ul class="list-disc pl-6 mt-2 space-y-1">
          <div class="flex items-center gap-2">
            <li>Screen recordings</li>
          </div>
          <div class="flex items-center gap-2">
            <li>Mouse movements and keyboard actions</li>
          </div>
          <div class="flex items-center gap-2">
            <li>Window layout metadata and accessibility tree data</li>
          </div>
          <li>Cryptocurrency wallet public key (for receiving rewards)</li>
        </ul>
      </section>

      <section>
        <h3 class="text-lg font-semibold mb-2">Data Usage</h3>
        <p>By uploading your recordings, you acknowledge and agree that:</p>
        <ol class="list-decimal pl-6 mt-2 space-y-2">
          <li>Your submitted recordings will be used to train artificial intelligence models</li>
          <li>Your data will be used in two ways:
            <div class="mt-2 space-y-4">
              <div>
                <h4 class="font-medium">Internal Usage:</h4>
                <ul class="list-disc pl-6 space-y-1">
                  <li>Stored in our internal datasets with associated wallet public keys</li>
                  <li>Used to train AI models</li>
                  <li>Never shared publicly in raw form</li>
                  <li>Model weights trained on this data may be released publicly</li>
                </ul>
              </div>
              <div>
                <h4 class="font-medium">Public Datasets:</h4>
                <ul class="list-disc pl-6 space-y-1">
                  <li>Processed to attempt to remove personally identifiable information (PII)</li>
                  <li>Wallet public keys removed for anonymization</li>
                  <li>Released under the MIT License</li>
                  <li>Published on Hugging Face</li>
                  <li>Available for commercial and non-commercial use</li>
                </ul>
              </div>
            </div>
          </li>
        </ol>
      </section>

      <section>
        <h3 class="text-lg font-semibold mb-2">Data Protection</h3>
        <p>We implement several measures to protect your privacy:</p>
        <div class="mt-4 space-y-4">
          <div>
            <h4 class="font-medium">Blocking and Filtering Tools:</h4>
            <ul class="list-disc pl-6 space-y-2">
              <li>
                <strong>Application and Window Blocking:</strong>
                <ul class="list-disc pl-6 mt-1">
                  <li>Block specific applications or windows from recording</li>
                  <li>Any data recorded while a blocked window is active is automatically excluded</li>
                  <li>This includes all associated keyboard input, mouse movements, and metadata during those time periods</li>
                </ul>
              </li>
              <li>
                <strong>AI-Powered Semantic Filtering:</strong>
                <ul class="list-disc pl-6 mt-1">
                  <li>Specify topics or keywords you want to block</li>
                  <li>Our AI automatically detects and censors any recording segments that are semantically related to your blocked topics</li>
                  <li>This helps prevent accidental capture of sensitive content even if expressed in different words</li>
                </ul>
              </li>
            </ul>
          </div>
        </div>
      </section>
    </div>
  </div>
</div>
