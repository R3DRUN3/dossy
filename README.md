# DOSSY  
[![Release](https://github.com/R3DRUN3/dossy/actions/workflows/release.yml/badge.svg)](https://github.com/R3DRUN3/dossy/actions/workflows/release.yml) 
[![Latest Release](https://img.shields.io/github/v/release/r3drun3/dossy?logo=github)](https://github.com/r3drun3/dossy/releases/latest)   
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)  ![Red Team Badge](https://img.shields.io/badge/Team-Red-red)  

<img src="./media/logo.png" width="300"/>

<br/>

**DOSSY** is a lightweight CLI for authorized volumetric Layer 7 HTTP stress testing:  
volumetric DoS, throughput flooding, connection exhaustion.     

Unlike synthetic benchmarking tools like `wrk` or `hey`, DOSSY simulates realistic adversarial traffic:  
randomized clients, mixed HTTP methods, varied request bodies and paths, making it purpose-built for red teamers.  

<br/>
For a deep dive on how Red Teams can build distributed infrastructure to simulate DDoS attacks, read [this article](https://www.neteye-blog.com/2025/02/building-a-distributed-ddos-infrastructure-for-red-teaming-campaigns/):  
**DOSSY is an excelent candidate for this type of activities !**    

<br/>

---

> [!CAUTION]
> **Use this tool only on systems you own or are authorized to test.**  
>  Unauthorized denial-of-service, service degradation, or traffic flooding against third-party systems is illegal and prohibited.  
> Any unlawful or harmful misuse is solely the responsibility of the user; developers and contributors bear no liability.  



## Features  
- Multiple target support
- Multiple HTTP methods support (GET, POST, PUT, DELETE, PATCH, OPTIONS)
- User-Agent rotation (35+ real-world UAs)
- Randomized path suffixes
- Optional Custom request body (`--body`) with configurable `--content-type`
- Latency tracking 
- Report export to `.json` or `.csv` (`--output`) for run diffing and regression tracking
- Pipelined async workers  


## Installation  
The recommended installation method is through the [latest release binary](https://github.com/R3DRUN3/dossy/releases/latest).  
You can also install the tool via [docker](https://github.com/R3DRUN3/dossy/pkgs/container/dossy).  

## Usage  

```bash
# Help
dossy -h

# Basic flood against a single target
dossy -t https://your-target.xyz -d 30

# Multiple targets, save a JSON report for later diffing
dossy -t https://your-target-1.xyz https://your-target-2.xyz -d 60 --output report.json

# CI mode (no progress bar) with CSV output
dossy -t https://example.com -d 30 --quiet --output report.csv

# POST with a JSON body
dossy -t https://your-target.xyz/api -d 30 --body '{"id":1}' --content-type application/json
```

[video-demo.webm](https://github.com/user-attachments/assets/e25236a9-d9e9-4a50-b45a-19e39bb50c65)




