import socket
import time
import threading
import random

HOST = '127.0.0.1'
PORT = 37520
FRAME = '\r\n'
SEP = ' = '

outputs = {
    'LampStart': '0',
    'LampLeader': '0',
    'LampRed': '0',
    'LampGreen': '0',
    'LampBlue': '0',
    'WooferLEDRed': '0',
    'WooferLEDGreen': '0',
    'WooferLEDBlue': '0',
    'SideLEDRed': '0',
    'SideLEDGreen': '0',
    'SideLEDBlue': '0',
    'ItemLEDRed': '0',
    'ItemLEDGreen': '0',
    'ItemLEDBlue': '0',
    'Billboard Red': '0',
    'Billboard Green': '0',
    'Billboard Blue': '0',
    'TicketCounter': '0',
    'TicketJackpot': '0',
    'Coin1': '0',
    'Coin2': '0',
    'HighScore': '0',
}

def send_all(client):
    client.send(('mame_start' + SEP + 'SonicDashExtreme' + FRAME).encode())
    for name, val in outputs.items():
        line = name + SEP + val + FRAME
        client.send(line.encode())

def set_and_send(client, name, val):
    outputs[name] = str(val)
    line = name + SEP + str(val) + FRAME
    client.send(line.encode())

def flash_lamps(client, names, repeats=3, delay=0.15):
    for _ in range(repeats):
        for n in names:
            on = '1' if outputs.get(n) == '0' else '0'
            set_and_send(client, n, on)
        time.sleep(delay)

def set_many(client, pairs):
    for n, v in pairs:
        set_and_send(client, n, v)

def attract_mode(client):
    print('[ATTRACT] Idle with blinking lamps...')
    set_many(client, [
        ('LampStart', '0'), ('LampLeader', '0'),
        ('LampRed', '0'), ('LampGreen', '0'), ('LampBlue', '0'),
        ('WooferLEDRed', '0'), ('WooferLEDGreen', '0'), ('WooferLEDBlue', '0'),
        ('SideLEDRed', '0'), ('SideLEDGreen', '0'), ('SideLEDBlue', '0'),
        ('ItemLEDRed', '0'), ('ItemLEDGreen', '0'), ('ItemLEDBlue', '0'),
        ('Billboard Red', '0'), ('Billboard Green', '0'), ('Billboard Blue', '0'),
        ('TicketCounter', '0'), ('TicketJackpot', '0'),
        ('Coin1', '0'), ('Coin2', '0'), ('HighScore', '0'),
    ])
    for _ in range(3):
        set_many(client, [('Billboard Red', '1'), ('Billboard Blue', '1')])
        time.sleep(0.8)
        set_many(client, [('Billboard Red', '0'), ('Billboard Blue', '0'),
                          ('Billboard Green', '1'), ('LampGreen', '1')])
        time.sleep(0.8)
        set_many(client, [('Billboard Green', '0'), ('LampGreen', '0')])
    # coin insert
    print('[ATTRACT] Coin inserted!')
    set_and_send(client, 'Coin1', '1')
    time.sleep(0.3)
    set_and_send(client, 'Coin2', '1')
    time.sleep(0.5)

def gameplay(client):
    print('[GAME] Starting...')
    set_many(client, [
        ('LampStart', '1'), ('LampLeader', '0'),
        ('LampGreen', '1'), ('SideLEDGreen', '1'),
        ('Billboard Green', '1'),
    ])
    time.sleep(1.0)
    # section 1
    print('[GAME] Section 1...')
    set_many(client, [
        ('WooferLEDRed', '1'), ('WooferLEDGreen', '1'),
        ('SideLEDBlue', '1'), ('ItemLEDGreen', '1'),
    ])
    time.sleep(2.0)
    # section 2
    print('[GAME] Section 2...')
    set_and_send(client, 'LampLeader', '1')
    set_and_send(client, 'SideLEDRed', '1')
    set_and_send(client, 'ItemLEDRed', '1')
    set_and_send(client, 'TicketCounter', '5')
    time.sleep(1.5)
    # section 3
    print('[GAME] Section 3...')
    set_and_send(client, 'SideLEDGreen', '0')
    set_and_send(client, 'SideLEDBlue', '1')
    set_and_send(client, 'WooferLEDRed', '0')
    set_and_send(client, 'WooferLEDBlue', '1')
    set_and_send(client, 'Billboard Blue', '1')
    set_and_send(client, 'Billboard Green', '0')
    set_and_send(client, 'TicketCounter', '12')
    time.sleep(2.0)
    # boss
    print('[GAME] BOSS FIGHT!')
    set_many(client, [
        ('LampRed', '1'), ('LampBlue', '1'),
        ('SideLEDRed', '1'), ('SideLEDGreen', '1'), ('SideLEDBlue', '1'),
        ('Billboard Red', '1'), ('Billboard Green', '1'), ('Billboard Blue', '1'),
        ('WooferLEDRed', '1'), ('WooferLEDGreen', '1'), ('WooferLEDBlue', '1'),
        ('ItemLEDRed', '1'), ('ItemLEDGreen', '1'), ('ItemLEDBlue', '1'),
        ('LampLeader', '1'),
    ])
    for i in range(10, 100, 15):
        set_and_send(client, 'TicketCounter', str(i))
        flash_lamps(client, ['LampRed', 'LampBlue', 'LampGreen'], repeats=1, delay=0.1)
        time.sleep(0.5)
    set_and_send(client, 'TicketCounter', '100')
    time.sleep(0.5)

def ticket_payout(client):
    print('[PAYOUT] Ticket payout!')
    set_and_send(client, 'TicketJackpot', '500')
    time.sleep(0.5)
    set_and_send(client, 'HighScore', '1250000')
    time.sleep(0.5)
    # rain tickets down
    for t in range(100, -1, -10):
        set_and_send(client, 'TicketCounter', str(t))
        time.sleep(0.3)
    set_and_send(client, 'TicketCounter', '0')
    time.sleep(0.3)
    set_and_send(client, 'TicketJackpot', '0')
    time.sleep(1.0)

def reset(client):
    print('[RESET] Back to attract...')
    for name in outputs:
        outputs[name] = '0'
    time.sleep(0.5)

def scenario(client):
    send_all(client)
    time.sleep(0.5)
    while True:
        attract_mode(client)
        gameplay(client)
        ticket_payout(client)
        reset(client)

def main():
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind((HOST, PORT))
    server.listen(1)
    print(f'[*] TCP simulator listening on {HOST}:{PORT}')
    print('[*] Launch WinGame to connect...')
    while True:
        client, addr = server.accept()
        print(f'[+] Client connected: {addr}')
        t = threading.Thread(target=scenario, args=(client,), daemon=True)
        t.start()

if __name__ == '__main__':
    main()
