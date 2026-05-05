# DOSSY  
[![Release](https://github.com/R3DRUN3/dossy/actions/workflows/release.yml/badge.svg)](https://github.com/R3DRUN3/dossy/actions/workflows/release.yml) 
[![Latest Release](https://img.shields.io/github/v/release/r3drun3/dossy?logo=github)](https://github.com/r3drun3/dossy/releases/latest)   
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)  ![Red Team Badge](https://img.shields.io/badge/Team-Red-red)  

<img src="./media/dossy_logo.png" width="300"/>

DOSSY is a lightweight CLI for authorized volumetric Layer 7 HTTP stress testing, particularly suited for DoS simulations.  
See the `features` paragraph for a list of capabilities.   
For a deep dive on how Red Teams can build distributed infrastructure to simulate DDoS attacks, read [this article](https://www.neteye-blog.com/2025/02/building-a-distributed-ddos-infrastructure-for-red-teaming-campaigns/):  
`dossy` is an excelent candidate for this type of activities !    

<br/>


> [!CAUTION]
> **Use this tool only on systems you own or are authorized to test.**  
>  Unauthorized denial-of-service, service degradation, or traffic flooding against third-party systems is illegal and prohibited.  
> Any unlawful or harmful misuse is solely the responsibility of the user; developers and contributors bear no liability.  



## Features  
- Multiple target support
- Multiple HTTP methods support
- User-Agent rotation
- Randomized path suffixes
- Latency tracking

## Installation  
You can download the [latest release binary](https://github.com/R3DRUN3/dossy/releases/latest), or install it via [docker](https://github.com/R3DRUN3/dossy/pkgs/container/dossy).  

## Usage  

```bash
dossy -h
```

[video-demo.webm](https://github.com/user-attachments/assets/e25236a9-d9e9-4a50-b45a-19e39bb50c65)




