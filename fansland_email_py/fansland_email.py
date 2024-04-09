import time
import redis

from lib.sendemail import SendEmail, SendEmailMsg
from botocore.exceptions import ClientError
import traceback

def main():
    r = redis.Redis(host='localhost', port=6379, db=0, password='gooDluck4u')
    email_client = SendEmail()

    chainid_maps = {
        # 主网
        '137': 'Polygon PoS',
        '56': 'BNB Smart Chain',
        '1': 'Ethereum',
        '42161': 'Arbitrum One',
        '10': 'OP Mainnet',
        '43114': 'Avalanche C-Chain',
        '1101': 'Polygon zkEVM',
        '204': 'opBNB',
        '8453': 'Base',
        '59144': 'Linea',


        # 测试网
        '80001': 'Polygon Mumbai Testnet',
        '97': 'BSC Testnet',
        '11155111': 'Ethereum Sepolia Testnet',
        '421614': 'Arbitrum Sepolia Testnet',
        '11155420': 'Op Sepolia Testnet',
        '43113': 'Avalanche Fuji Testnet',
        '1442': 'Polygon zkEVM Testnet',
        '5611': 'opBNB Testnet',
        '84532': 'Base Sepolia Testnet',
        '59140': 'Linea Goerli Testnet',

        # solana
        'solona': 'Solana Mainnet',
        'devnet': 'Solana Devnet',
    }

    # TODO: 修改成正式的票名
    typeid_maps = {
        '0': "Early Bird 2-Day Ticket (4-5 May)",
        '1': "Advance 2-Day Ticket (4-5 May)",
        '2': "Regular 1-Day Ticket (4 May)",
        '3': "Regular 1-Day Ticket (5 May)",
        '4': "Regular 2-Day Ticket (4-5 May)",
    }


    while True:
        print('=================start send email ===============')
        save_back = ''
        try:
            raw_msg = r.lpop('sendemail')
            if raw_msg is  None:
                print('empty email queue')
            else:
                save_back = raw_msg.decode('utf-8')
                items = raw_msg.decode('utf-8').split(';')
                chainid = items[0]
                address = items[1]
                token_id = items[2]
                type_id = items[3]
                qrcode_txt = items[4]

                # 修复重复发送邮件
                only_once_email_key = f'email:{chainid}:{token_id}:{str(address).lower()}'
                print('唯一邮件key: {}'.format(only_once_email_key))
                ret =r.get(only_once_email_key)
                if ret is not None:
                    if ret.decode('utf-8') == user_email:
                        print("此key已存在, 不再重复发送, key:  {}".format(only_once_email_key))
                        continue

                # 查一下redis该tokenid的最新owner
                user_address = r.get('nft:{}:nft:tokenid:owner:{}'.format(chainid, token_id))
                if user_address is None:
                    print("token_id {} owner is empty".format(token_id))
                    continue

                if user_address.decode('utf-8').lower() != str(address).lower():
                    print("token_id {} owner is not matched, expected: {}, got: ".format(
                        token_id, address, user_address.decode('utf-8').lower() ))
                    pass

                user_email = r.get('bindemail:{}'.format(address))
                if user_email is None:
                    print('address {} bind email is empty'.format(address))
                    continue
                user_email = user_email.decode('utf-8')

                chain = chainid_maps[chainid]
                print(user_email)

                response = email_client.send_raw_email(SendEmailMsg(
                        to_email=user_email,
                        qrcodes=[qrcode_txt],
                        address=address,
                        chain=chain,
                        chainid=chainid,
                        ticket_type = typeid_maps[str(type_id)],
                        token_id=token_id
                    ))

                print("Email sent! Message ID:"),
                print(response['MessageId'])

                # 发送完邮件, 设置一下key
                ret = r.set(only_once_email_key, user_email)
                print('set key: {}, ret:{}'.format(only_once_email_key, ret))
        # except ClientError as e:
        #     print(e.response['Error']['Message'])
        except Exception as e:
            print("errror=========\n{}".format(e))
            # 放回去队列尾部
            r.rpush('sendemail', save_back)
            save_back = ''
            traceback.print_exc()

        time.sleep(5)

    pass


if __name__ == '__main__':
    main()

