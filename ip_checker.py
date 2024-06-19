import subprocess
import threading
import sqlite3
from ipaddress import ip_address


class DBConn():
    def __init__(self, db_name="valid_ips.db"):
        self.con = self.set_db_conn(db_name)
        self.cur = self.set_cur()
    
    @staticmethod
    def set_db_conn(db_name):
        return sqlite3.connect(db_name)
    
    def close_con(self):
        self.con.close()

    def set_cur(self):
        cur = self.con.cursor()
        cur.execute("CREATE TABLE IF NOT EXISTS ip(addr, UNIQUE(addr))")
        cur.execute("CREATE TABLE IF NOT EXISTS checked(addr, UNIQUE(addr))")
        cur.execute(f"""INSERT OR IGNORE INTO checked VALUES
                    ('0')""")
        return cur

    def put_valid_ip(self, ip):
        cur = self.cur
        cur.execute(f"""INSERT OR IGNORE INTO ip VALUES
                    ('{ip}')""")
        self.con.commit()

    def put_checked_ip(self, ip):
        cur = self.cur
        cur.execute(f"""INSERT OR IGNORE INTO checked VALUES
                    ('{ip}')""")
        self.con.commit()
        
    def return_db(self):
        res = self.cur.execute("SELECT addr FROM ip")
        return res.fetchall()
    
    def return_last_checked_ip(self):
        res = self.cur.execute("SELECT * FROM checked WHERE ROWID IN ( SELECT max( ROWID ) FROM checked )")
        return res.fetchall()[0][0] #data: [('foo',)]
        

class IPChecker():
    def __init__(self):
        self.max_ip_int = self.ip_as_int('255.255.255.255')
        self.reserved_ranges = self.build_special_address_blocks()
        self.valid_ips = []

    def start(self):
        last = int(db.return_last_checked_ip())
        threads = 50
        ts = []
        while (last < self.max_ip_int):
            
            if block := self.check_block(last):
                last = block[1] + 1

            
            for i in range(threads):
                   bar = self.ip_from_int(last+i)
                   foo = threading.Thread(target=self._check_ip, args=(bar,))
                   ts.append(foo)

            for j in ts:
                j.start()

            for x in ts:
                x.join()
            
            for i in range(last, last+threads):
                self._store_checked_ip(i)

            for i in self.valid_ips:
                db.put_valid_ip(self.ip_as_int(i))
            
            last += threads
            ts = []
            self.valid_ips = []

    def _store_valid_ip(self, ip):
        print(f'valid ip {ip} found')
        self.valid_ips.append(ip) 

    def _store_checked_ip(self, ip):
        db.put_checked_ip(self.ip_as_int(ip))      
    
    def _check_ip(self, ip):
        print(f"pinging {ip}")
        call = subprocess.run(f"ping -c 1 {ip}", shell=True, stdout=subprocess.DEVNULL)
        if call.returncode == 0:
            self._store_valid_ip(ip)
            return True

    def check_block(self, ip_int):
        for i in self.reserved_ranges:
            if ip_int >= i[0]:
                if ip_int <= i[1]:
                    if i == self.reserved_ranges[-1]:
                        # were done!
                        return False
                    return i
        return False
        
    def build_special_address_blocks(self):
           return [
            (self.ip_as_int('0.0.0.0'), self.ip_as_int('0.255.255.255')),
            (self.ip_as_int('10.0.0.0'), self.ip_as_int('10.255.255.255')),
            (self.ip_as_int('100.64.0.0'), self.ip_as_int('100.127.255.255')),
            (self.ip_as_int('127.0.0.0'), self.ip_as_int('127.255.255.255')),
            (self.ip_as_int('169.254.0.0'), self.ip_as_int('169.254.255.255')),
            (self.ip_as_int('172.16.0.0'), self.ip_as_int('172.31.255.255')),
            (self.ip_as_int('192.0.0.0'), self.ip_as_int('192.0.0.255')),
            (self.ip_as_int('192.0.2.0'), self.ip_as_int('192.0.2.255')),
            (self.ip_as_int('192.88.99.0'), self.ip_as_int('192.88.99.255')),
            (self.ip_as_int('192.168.0.0'), self.ip_as_int('192.168.255.255')),
            (self.ip_as_int('198.18.0.0'), self.ip_as_int('198.19.255.255')),
            (self.ip_as_int('198.51.100.0'), self.ip_as_int('198.51.100.255')),
            (self.ip_as_int('203.0.113.0'), self.ip_as_int('203.0.113.255')),
            (self.ip_as_int('224.0.0.0'), self.ip_as_int('239.255.255.255')),
            (self.ip_as_int('233.252.0.0'), self.ip_as_int('233.252.0.255')),
            (self.ip_as_int('240.0.0.0'), self.ip_as_int('255.255.255.255'))
            ]   
    
    @staticmethod
    def ip_as_int(ip):
        return int(ip_address(ip).packed.hex(), 16)
    
    @staticmethod
    def ip_from_int(ip_int):
        return ip_address(ip_int)
    
    @staticmethod
    def _convert_to_str(ip_data):
        ip = [str(i) for i in ip_data]
        return '.'.join(ip)


if __name__=='__main__':
    db = DBConn()
    checker = IPChecker()
    checker.start()
