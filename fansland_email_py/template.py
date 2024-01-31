

def get_mint_template(address: str, img_count: int) :
    first_part = f"""<div style=width:100%; background-color:#E4E5E7;padding:2px;>
        <div style=max-width:640px; margin:0 auto; background:#ffffff;border-radius:10px; overflow:hidden;>
            <img src=https://static.fansland.io/email/20240126-114328.jpg style=width:100%  alt='' />
            <div style=padding:32px;>
                <h1 style=font-size:24px; font-weight:bold; text-align:center; color:#081131;> Fansland NFT ticket Mint success! 🎉 </h1>
                <p style=margin:0;><br></p>
                <p style=font-size:16px; line-height:170%; letter-spacing:0.2px;  margin:0;> Congratulations! 👏</p>
                <p style=margin:0;><br></p>
                <p style=font-size:16px; line-height:170%; letter-spacing:0.2px; margin:0;> You have successfully mint Fansland NFT tickets with your wallet address: </p>
                <p style=margin:0;><br></p>
                <p style=font-size:16px; text-align:center;  line-height:170%; letter-spacing:0.2px; margin:0;> {address} </p>
                <p style=margin:0;><br></p>
                <p style=font-size:16px; line-height:170%; letter-spacing:0.2px; margin:0;>
                    Please login to <a href="https://fansland.io/">https://fansland.io</a> with this wallet address to see your NFT tickets.
                </p>
                <p style=margin:0;><br></p>
                <hr>
                <p style=font-size:14px; font-style: italic;  text-align:center;><em> Fansland aims to build the best Web3 infrastructure for global fans.<em> </p>
                """

    img_part = ""
    for i in range(img_count):
        img_part += f"""<img src="cid:qrcode{i}" alt="qrcode{i}">"""
        img_part += f"""<p style=margin:0;><br></p>"""

    last_part = """
            </div>
        </div>
    </div>"""

    return first_part + img_part + last_part