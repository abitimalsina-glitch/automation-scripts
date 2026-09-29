#!/bin/bash

ROUTE=$(route get default 2>/dev/null)

INTERFACE=$(echo "$ROUTE" | awk '/interface:/ {print $2}')
GATEWAY=$(echo "$ROUTE" | awk '/gateway:/ {print $2}')
LOCAL_IP=$(ipconfig getifaddr "$INTERFACE")
PUBLIC_IP=$(curl -4 -s --max-time 5 ifconfig.me)

ping_test() {
    RESULT=$(ping -c 3 "$1" 2>/dev/null)

    if [ $? -ne 0 ]; then
        echo "Status       OFFLINE"
        return
    fi

    STATS=$(echo "$RESULT" | tail -1 | cut -d= -f2)
    LOSS=$(echo "$RESULT" | grep -Eo '[0-9.]+% packet loss' | awk '{print $1}')

    echo "Status       ONLINE"
    echo "Packet Loss  $LOSS"
    echo "Latency      $(echo "$STATS" | cut -d/ -f2) ms avg"
    echo "Min / Max    $(echo "$STATS" | cut -d/ -f1) / $(echo "$STATS" | cut -d/ -f3) ms"
    echo "Jitter       $(echo "$STATS" | cut -d/ -f4 | awk '{print $1}') ms"
}

echo "NETWORK"
echo "──────────────────────────────────────"
echo "Interface    $INTERFACE"
echo "Local IP     $LOCAL_IP"
echo "Gateway      $GATEWAY"
echo "Public IP    $PUBLIC_IP"

echo
echo "INTERFACE"
echo "──────────────────────────────────────"
echo "MAC Address  $(ifconfig "$INTERFACE" | awk '/ether/ {print $2}')"
echo "Status       $(ifconfig "$INTERFACE" | awk '/status:/ {print $2}')"

echo
echo "GATEWAY"
echo "──────────────────────────────────────"
ping_test "$GATEWAY"

echo
echo "INTERNET"
echo "──────────────────────────────────────"
ping_test "1.1.1.1"

echo
echo "DNS"
echo "──────────────────────────────────────"
scutil --dns | awk '/nameserver\[[0-9]+\]/ {print "Nameserver   " $3}' | sort -u

echo
echo "DNS TEST"
echo "──────────────────────────────────────"

DNS=$(dig +short google.com | head -1)

if [ -n "$DNS" ]; then
    echo "Status       WORKING"
    echo "google.com   $DNS"
else
    echo "Status       FAILED"
fi

echo
echo "HTTPS"
echo "──────────────────────────────────────"

HTTP=$(curl -s -o /dev/null -w "%{http_code}" --max-time 5 https://google.com)

if [ "$HTTP" != "000" ]; then
    echo "Status       ONLINE"
    echo "HTTP Code    $HTTP"
else
    echo "Status       OFFLINE"
fi
