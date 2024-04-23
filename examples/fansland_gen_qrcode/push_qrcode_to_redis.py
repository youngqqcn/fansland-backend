import redis

def read_qrcodes(path, nums):
    qrcodes = []

    with open(path) as infile:
        lines = infile.readlines()
        for qrcode in lines:
            qrcode = qrcode.strip()
            if qrcode == '':
                print('跳过空行')
                continue
            qrcodes.append(qrcode)

            if len(qrcodes) >= nums:
                break
    return qrcodes


def push_to_redis(path, ticket_type_id, nums):
    print('=======================')
    r = redis.Redis(host='localhost', port=6379, db=0, password='gooDluck4u')
    qrcodes = read_qrcodes(path, nums)
    assert len(qrcodes) > 0 , "empty file"
    # if True:
    for w in qrcodes:
        # r.sadd("whitelists:advance:0410",  w)
        r.rpush('redeempool:{}'.format(ticket_type_id), w)
        # print('插入成功: {}'.format(w))
    print('一共{}'.format(len(qrcodes)))
    pass

def main():
    push_to_redis('./qrcodes_04_23/qrcode_early.csv', 0, 3000)
    push_to_redis('./qrcodes_04_23/qrcode_advance.csv', 1, 3000)
    push_to_redis('./qrcodes_04_23/qrcode_regular_4_5.csv', 2, 3000)
    push_to_redis('./qrcodes_04_23/qrcode_regular_5_5.csv', 3, 3000)
    push_to_redis('./qrcodes_04_23/qrcode_regular_2_days.csv', 4, 3000)
    pass

if __name__ == '__main__':
    main()

