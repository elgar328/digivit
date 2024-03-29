# digivit

<img width="250" alt="KAMAN digiVIT" src="https://github.com/elgar328/digivit/assets/93251045/ddc88b36-6fd5-4be0-847c-008ee3b6517d">

Data logger for KAMAN digiVIT

- Distance data is periodically collected from digiVIT via Ethernet port and stored in a file.

- It is designed to work with a single digiVIT device and is not intended for multiple devices.

- A Distance Output of 100,000 corresponds to 100% of the measurement range.

- Use the default IP address and ports for digiVIT.

> Default IP Address: 192.168.0.145
>
> Default UDP Writer Port: 55555
>
> Default UDP Reader Port: 55556

- It is available in various network connections as shown below.

> **computer**←-------wireless-------→**router**←-------wire-------→**digiVIT**
>
> **computer**←-------wire-------→**router**←-------wire-------→**digiVIT**
>
> **computer**←-------crossed wire-------→**digiVIT**

- To directly connect the computer and digiVIT with a crossed wire, set the Internet Protocol (TCP/IP) properties of the computer as follows:

> IP address : 192.168.0.1
>
> Subnet mask : 255.255.255.0
>
> Default gateway : 192.168.0.145

- If you encounter a lot of missing data, use a slower sampling rate.

<img src="https://user-images.githubusercontent.com/93251045/223617407-1061718d-a5c4-41ee-8327-a6ef83d446de.png"  width="700">
