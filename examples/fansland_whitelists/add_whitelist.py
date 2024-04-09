import redis

def read_whitelist_address():
    whitelists = []
    with open('whitelists.txt') as infile:
        lines = infile.readlines()
        for addr in lines:
            a = addr.lower().strip()
            if a == '':
                print('跳过空行')
                continue

            assert len(a) == 42 and a.startswith('0x') , "invalid address {}".format(a)
            whitelists.append(a)
    return whitelists



def main():
    r = redis.Redis(host='localhost', port=6379, db=0, password='gooDluck4u')
    whitelists = read_whitelist_address()
    assert len(whitelists) > 0 , "empty whitelist"
    for w in whitelists:
        r.sadd("whitelists:advance:0410",  w)
        print('插入成功: {}'.format(w))
    pass

if __name__ == '__main__':
    main()

