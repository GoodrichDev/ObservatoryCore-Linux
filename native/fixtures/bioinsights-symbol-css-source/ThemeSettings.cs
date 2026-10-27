namespace EDJP.Observatory;

internal class ThemeSettings
{
	public string? ThemePath { get; set; }

	public int? BorderWidth { get; set; }

	public int? RowSpacing { get; set; }

	public bool UseHighlightRow { get; set; }

	public int RowHeight { get; set; }

	public ThemeSettings()
	{
		while (true)
		{
			int num = -1052325989;
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ -220124432)) % 3)
				{
				case 2u:
					break;
				case 1u:
					goto IL_0028;
				default:
					RowHeight = 20;
					return;
				}
				break;
				IL_0028:
				UseHighlightRow = false;
				num = ((int)num2 * -648539005) ^ 0xF7E4BE2;
			}
		}
	}
}
