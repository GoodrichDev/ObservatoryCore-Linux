using System.Drawing;
using System.Runtime.CompilerServices;

namespace EDJP.Observatory;

public class ThemeColor
{
	public int R { get; set; }

	public int G { get; set; }

	public int B { get; set; }

	public ThemeColor()
	{
	}

	public ThemeColor(int r, int g, int b)
	{
		while (true)
		{
			int num = -195948265;
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ -1937553223)) % 5)
				{
				case 0u:
					break;
				default:
					return;
				case 4u:
					B = b;
					num = (int)((num2 * 844689485) ^ 0x2F83C1E6);
					continue;
				case 3u:
					G = g;
					num = (int)(num2 * 1513264344) ^ -1085780559;
					continue;
				case 2u:
					R = r;
					num = (int)((num2 * 201186520) ^ 0x7664EF4E);
					continue;
				case 1u:
					return;
				}
				break;
			}
		}
	}

	public string ToHex()
	{
		return ToHex(showHash: false);
	}

	[MethodImpl(MethodImplOptions.NoInlining)]
	public string ToHex(bool showHash)
	{
		return _202A_206F_200B_200F_206F_200E_200F_200B_200F_200D_202D_206A_200C_200C_206B_202C_202D_206C_200F_202E_206E_200E_200D_202D_200D_202A_200F_202D_202A_206A_200E_200B_202C_200B_200F_202A_200C_202D_202D_200E_202E(showHash ? "#" : "", R.ToString("X2"), G.ToString("X2"), B.ToString("X2"));
	}

	public Color ToColor()
	{
		return Color.FromArgb(R, G, B);
	}

	static string _202A_206F_200B_200F_206F_200E_200F_200B_200F_200D_202D_206A_200C_200C_206B_202C_202D_206C_200F_202E_206E_200E_200D_202D_200D_202A_200F_202D_202A_206A_200E_200B_202C_200B_200F_202A_200C_202D_202D_200E_202E(string P_0, string P_1, string P_2, string P_3)
	{
		return P_0 + P_1 + P_2 + P_3;
	}
}
