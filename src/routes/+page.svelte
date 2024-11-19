<script>
    import {
       onMount
   } from 'svelte';

   let totalCores;
   let cores = 3;
   let isMining = false;
   let bestscore = 0;
   let allResults = [];
   const LOOPS = 100000;

   let hashPerSecond = 0;

   /**
   * @type {number | undefined}
   */
   let intervalCount;
   /**
   * @type {string}
   */
   let wallet;

   function increaseCores() {
       if (isMining) return;
       cores = Math.min(16, cores + 1);
   }

   function decreaseCores() {
       if (isMining) return;
       cores = Math.max(1, cores - 1);
   }

   /**
    * Checks if the given string is an address
    *
    * @method isAddress
    * @param {String} address the given HEX adress
    * @return {Boolean}
    */
   function isAddress(address) {
       if (!/^(0x)?[0-9a-f]{40}$/i.test(address)) {
           // check if it has the basic requirements of an address
           return false;
       }
       // no checksum
       return (/^(0x)?[0-9a-f]{40}$/i.test(address));
   };

   onMount(() => {
       totalCores = navigator.hardwareConcurrency;
       cores = Math.ceil(totalCores / 2);

       wallet = String(localStorage.getItem('wallet') || '')
          
        if (wallet && !isAddress(wallet)) {
            wallet = '';
        }
        bestscore = Number(localStorage.getItem('bestscore-'+wallet)) || 0;
        
        try {
            allResults = JSON.parse(localStorage.getItem('allResults-'+wallet)) || [];
        } catch (e) {
            allResults = [];
        }
   })


/**
   * @type {any[]}
   */
   let workers = [];

   function doStop() {
       isMining = false;
       workers.forEach((w) => {
           w.worker.terminate();
           w.status = 'stop';
       });
       workers = [...workers];
       clearInterval(intervalCount);
   }

   function terminateWorkers() {
       doStop();
       workers = [];
   }

   let globalStart = 0;
   let globalElapsed = 0;

   function startMining() {
       terminateWorkers();

       intervalCount = setInterval(() => {
            hashPerSecond = workers.reduce((acc, w) => acc + w.hashPerSecond, 0);
        }, 1000);

       if (!wallet) {
           alert('Please enter a valid Ethereum address, on wallet field');
           return;
       }

       if (!isAddress(wallet)) {
           alert('Please enter a valid Ethereum address, on wallet field');
           wallet = '';
           return;
       }

       localStorage.setItem('wallet', wallet);

       isMining = true;
       cores = Math.min(cores, Math.min(navigator.hardwareConcurrency, 20));

       globalStart = +new Date();
       //setInterval(() => {
          // globalElapsed = +new Date() - globalStart;
       //}, 1000);

       for (let i = 0; i < cores; i++) {
           const worker = new Worker('/sw.js', {
               type: 'module'
           });
           workers[i] = {
               worker: worker,
               status: 'init',
               start: 0,
               end: 0,
               loops: 0,
               hashPerSecond: 0
           };

           let time;

            worker.onmessage = function(event) {
                const _workerStruct = workers[i];
                // console.log('Worker' + i, event.data.status);
                _workerStruct.status = event.data.status;
                worker.onmessage = async function(event) {
                    if (event.data.status === 'ready') {
                        _workerStruct.start = Date.now();
                        _workerStruct.loops = 0;
                        worker.postMessage({
                           wallet,
                           bestscore,
                           times: LOOPS
                        });
                        return;
                    }

                    // looping
                    //console.log('Worker' + i, event.data.results);
                    let recordBreak = false;
                    if(event.data.results.score > bestscore) {
                        bestscore = event.data.results.score;
                        localStorage.setItem('bestscore-'+wallet, String(bestscore));
                        allResults.push(event.data.results);
                        allResults = [...allResults];
                        console.log(allResults)
                        localStorage.setItem('allResults-'+wallet, JSON.stringify(allResults));
                        recordBreak = true;
                        if (bestscore < 70) {
                            recordBreak = false;
                        }
                    }

                    _workerStruct.loops += 1;
                    _workerStruct.hashPerSecond = Math.floor(
                        (_workerStruct.loops * LOOPS) / ((Date.now() - _workerStruct.start) / 1000)
                    );
                    //console.log('Worker' + i + ', hash/sec:', _workerStruct.hashPerSecond);
                    _workerStruct.end = 0;

                    if(recordBreak) {
                        doStop();
                        return;
                    } else {
                        worker.postMessage({
                            wallet,
                            bestscore,
                            times: LOOPS
                        });
                    }
                    //doStop();
                    //return;
                }
               

               // loop
               worker.postMessage({
                   wallet,
                   bestscore,
                   times: LOOPS
               });
           };

           worker
               .postMessage({
                   wallet,
                   bestscore,
                   times: LOOPS
               });
       };
       workers = [...workers];
   }
</script>

<style>
    :global(body) {
        @apply bg-gray-900 text-white
    }
</style>

  <div class="container mx-auto p-8">
    <header class="text-center mb-12">
      <h1 class="text-4xl font-bold mb-4">Uniswap v4 Address Mining Tool</h1>
      <p class="text-lg">Find a salt value to deploy Uniswap v4 at an optimal address using CREATE2.</p>
    </header>
    
 
    <section class="my-4 bg-gray-800 p-6 rounded-lg">

    
        <h2 class="text-2xl font-semibold mb-4">Submission Instructions</h2>
        <p class="mb-4">*Participants can submit as many unique addresses as they want during the challenge. To make sure that only you can submit your salt, set the first 20 bytes of your salt to the Ethereum address executing the submission. Alternatively, you can leave the first 20 bytes as 0 bytes, but your submission could be frontrun. The last 12 bytes of the salt can be anything you choose.</p>
        <div class="mb-4">
            <label for="wallet" class="block text-lg font-semibold mb-2">Your wallet address:</label>
            <input id="wallet" bind:value={wallet} type="text" class="w-full p-2 mb-1 rounded bg-gray-700 text-white" placeholder="Enter your Ethereum address">
            <label for="cores" class="block text-lg font-semibold mb-2">Number of Cores to Use:</label>
            <div class="flex items-center space-x-4">
            <button on:click={decreaseCores} class="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded">-</button>
            <input id="cores" type="number" min="1" max="32" disabled={isMining} value={cores} class="w-12 p-2 rounded bg-gray-700 text-white mx-auto text-center">
            <button on:click={increaseCores} class="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded">+</button>
          </div>
        </div>
        {#if !isMining}
            <button class="bg-pink-500 hover:bg-pink-600 text-white font-bold py-2 px-4 rounded" on:click={startMining}>Start Mining Now</button>
        {:else}
            <button class="bg-red-500 hover:bg-red-600 text-white font-bold py-2 px-4 rounded" on:click={doStop}>Stop Mining</button>
        
        {/if}
        {#if hashPerSecond}
        Hashes per second: {hashPerSecond}
        {/if}
    
        <hr class="m-2" />
        <h3 class="text-2xl font-semibold mb-4">IMPORTANT</h3>
        <p class="text-xl">
            Once you got a salt you can submit it using etherscan:
             <a target="_blank" class="underline hover:text-pink-600 text-pink-500" href="https://etherscan.io/address/0x48e516b34a1274f49457b9c6182097796d0498cb#writeContract#F2">0x48e516b34a1274f49457b9c6182097796d0498cb</a>,
              if you need more info read this blogpost: <a target="_blank" class="underline hover:text-pink-600 text-pink-500" href="https://blog.uniswap.org/uniswap-v4-address-mining-challenge">https://blog.uniswap.org/uniswap-v4-address-mining-challenge</a>
        </p>
    </section>
  
  
    <section class="my-4 bg-gray-800 p-6 rounded-lg">
      <h2 class="text-2xl font-semibold mb-4">Mining Results</h2>
      <div class="overflow-x-auto">
        <table class="min-w-full bg-gray-700 rounded-lg">
          <thead>
            <tr>
              <th class="py-3 px-6 text-left">Salt</th>
              <th class="py-3 px-6 text-left">Address</th>
              <th class="py-3 px-6 text-left">Score</th>
            </tr>
          </thead>
          <tbody>
            {#each allResults.sort((a,b) => b.score - a.score) as r}
            <!-- Example row -->
            <tr class="bg-gray-800 border-b border-gray-600">
              <td class="py-3 px-6">0x{r.salt}</td>
              <td class="py-3 px-6">0x{r.address}</td>
              <td class="py-3 px-6">{r.score}</td>
            </tr>
            {/each}
            <!-- Additional rows will be dynamically added here -->
          </tbody>
        </table>
      </div>
    </section>

    <section class="my-4 bg-gray-800 p-6 rounded-lg mb-8">
      <h2 class="text-2xl font-semibold mb-4">Challenge Details</h2>
      <p class="mb-4">The challenge will run from <strong>November 10th, 2024</strong> to <strong>December 1st, 2024</strong>. Uniswap v4 will be deployed using the CREATE2 function. This function generates deterministic addresses using:</p>
      <ul class="list-disc list-inside mb-4">
        <li>The hash of the initcode for Uniswap v4: <code class="bg-gray-700 p-1 rounded">0x94d114296a5af85c1fd2dc039cdaa32f1ed4b0fe0868f02d888bfc91feb645d9</code></li>
        <li>The deployer address for Uniswap v4: <code class="bg-gray-700 p-1 rounded">0x48E516B34A1274f49457b9C6182097796D0498Cb</code></li>
        <li>Your choice of a salt*</li>
      </ul>
      <p>By combining these elements, you can generate different candidate addresses. Participants can use tools like <strong>create2crunch</strong> to generate and submit salts, corresponding to candidate addresses, to the challenge smart contract on Ethereum mainnet.</p>
    </section>
    
    <section class="my-4 bg-gray-800 p-6 rounded-lg mb-8">

      <h2 class="text-2xl font-semibold mb-4">Scoring Criteria</h2>
      <ul class="list-disc list-inside">
        <li>10 points for each leading 0 nibble</li>
        <li>40 points if the address starts with four consecutive 4s</li>
        <li>20 points if the first nibble after the four 4s is not a 4</li>
        <li>20 points if the last four nibbles are all 4s</li>
        <li>1 point for each 4 elsewhere in the address</li>
      </ul>
    </section>
   
  </div>
