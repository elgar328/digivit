# digivit

Used to communicate with the KAMAN digiVIT via the ethernet port.

- It has only a simple function of periodically transmitting 'MD' command to digiVIT and collecting Distance Output. The rest of the work, such as editing settings, must be done directly in digiVIT.

- It is designed to collect data from a single digiVIT deviceand, and is not intended for multiple digiVITs. 

- A Distance Output of 100,000 corresponds to 100% of the measurement range.
- Use the default IP address and ports for digiVIT.

> Default IP Address: 192.168.0.145
>
> Default UDP Writer Port: 55555
>
> Default UDP Reader Port: 55556

- Since the process of requesting data from the computer to digiVIT is done through synchronous udp communication, the sample rate depends on the network environment.

> ≈ 30 Hz  @  (**computer**←-------wireless-------→**router**←-------wire-------→**digiVIT**)
>
> ≈ 50 Hz  @  (**computer**←-------wire-------→**router**←-------wire-------→**digiVIT**)
>
> ≈ 50 Hz  @  (**computer**←-------crossed wire-------→**digiVIT**)

- To directly connect the computer and digiVIT with a crossed wire, set the Internet Protocol (TCP/IP) properties of the computer as follows:

> IP address : 192.168.0.1
>
> Subnet mask : 255.255.255.0
>
> Default gateway : 192.168.0.145
>
> Preferred DNS server : 8.8.8.8
