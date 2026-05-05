# DOSSY  
[![Release](https://github.com/R3DRUN3/dossy/actions/workflows/release.yml/badge.svg)](https://github.com/R3DRUN3/dossy/actions/workflows/release.yml) 
[![Latest Release](https://img.shields.io/github/v/release/r3drun3/dossy?logo=github)](https://github.com/r3drun3/dossy/releases/latest)   
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)  ![Red Team Badge](https://img.shields.io/badge/Team-Red-red)  

<img src="./media/dossy_logo.png" width="300"/>

DOSSY is a lightweight CLI for authorized volumetric Layer 7 HTTP stress testing, particularly suited for DoS simulations.  
It can send concurrent HTTP requests against one or more target URLs, track throughput and latency, and print a live progress view plus a final summary.  
For a deep dive on how Red Teams can build infrastructure to simulate DDoS attacks, read [this article](https://www.neteye-blog.com/2025/02/building-a-distributed-ddos-infrastructure-for-red-teaming-campaigns/).  

<br/>


> [!CAUTION]
> **Use this tool only against systems you own or are explicitly authorized to test.**  
> Unauthorized denial-of-service activity, service degradation, or traffic flooding against third-party systems is illegal, unethical, and prohibited.  
> In the event that this tool is weaponized or otherwise used for unlawful, unauthorized, or harmful purposes, sole and exclusive responsibility  
> shall rest with the individuals or entities making such use, and under no circumstances shall 
> any liability be attributed to the developers, contributors, or maintainers of the tool.  

## Features  
- Multi-target distribution
- Multiple HTTP methods support
- User-Agent rotation
- Randomized path suffixes
- Latency tracking

## Installation  
You can download the [latest release binary](https://github.com/R3DRUN3/dossy/releases/latest), or install it via [docker](https://github.com/R3DRUN3/dossy/pkgs/container/dossy).  

## Usage  



