import requests

def is_user_in_group(bot_token, group_id, user_id):
    api_url = f"https://api.telegram.org/bot{bot_token}/getChatMember"
    params = {
        'chat_id': group_id,
        'user_id': user_id
    }
    response = requests.get(api_url, params=params)
    data = response.json()
    print(f'data:{data}')
    if data['ok']:
        if data['result']['status'] == 'left':
            return False
        else:
            return True
    else:
        # 处理 API 请求错误
        return False

# 在下面填入你的 Bot Token、群组 ID 和用户 ID
bot_token = "6709002095:AAG3zZpWpwTW_0sT1rscy2Li9C-LBnC6RO8"
# group_id = '-4152174278'
# group_id = '-1002052126216' # 测试
group_id = '-1002000666414'  # fansland
user_id = '2121363153'

result = is_user_in_group(bot_token, group_id, user_id)
print(result)