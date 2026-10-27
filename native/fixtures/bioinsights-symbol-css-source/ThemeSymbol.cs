namespace EDJP.Observatory;

public class ThemeSymbol
{
	public string Content { get; set; }

	public SymbolContentType ContentType { get; set; }

	public ThemeColor? ContentColor { get; set; }

	public string? ContentColorKey { get; set; }

	public ThemeColor? ContentBackColor { get; set; }

	public string? ContentBackColorKey { get; set; }

	public string Description { get; set; }

	public string? Font { get; set; }

	public int? FontSize { get; set; }

	public string? FontWeight { get; set; }

	public string? TextOverride { get; set; }

	public string? TextOverrideFont { get; set; }

	public ThemeColor? TextOverrideColor { get; set; }

	public string? TextOverrideColorKey { get; set; }

	public bool? SupressMouseOver { get; set; }

	public int? PaddingLeft { get; set; }

	public int? PaddingRight { get; set; }

	public int? PaddingTop { get; set; }

	public int? PaddingBottom { get; set; }

	public int? Width { get; set; }

	public string? VerticalAlign { get; set; }

	public bool? IsLabel { get; set; }

	public ThemeColor GetTextOverrideColor()
	{
		if (TextOverrideColor == null)
		{
			while (true)
			{
				int num = -1107816451;
				while (true)
				{
					uint num2;
					switch ((num2 = (uint)(num ^ -7146162)) % 5)
					{
					case 3u:
						break;
					case 0u:
						return ContentColor;
					case 2u:
						return new ThemeColor(255, 255, 255);
					case 1u:
					{
						int num3;
						int num4;
						if (ContentColor == null)
						{
							num3 = 1604707476;
							num4 = num3;
						}
						else
						{
							num3 = 1339515371;
							num4 = num3;
						}
						num = num3 ^ ((int)num2 * -835045613);
						continue;
					}
					default:
						goto end_IL_0008;
					}
					break;
				}
				continue;
				end_IL_0008:
				break;
			}
		}
		return TextOverrideColor;
	}

	public string GetTextOverride()
	{
		if (TextOverride == null)
		{
			while (true)
			{
				uint num;
				switch ((num = 2120519725u) % 3)
				{
				case 2u:
					continue;
				case 1u:
					return Content;
				}
				break;
			}
		}
		return TextOverride;
	}
}
