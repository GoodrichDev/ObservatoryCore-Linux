// Source AppendLine is expanded to Windows CRLF for cross-platform capture.
using ListViewSubItem=System.Windows.Forms.ListViewItem.ListViewSubItem;
using System;
using System.IO;
using System.IO.Compression;
using System.Text;
using System.Windows.Forms;
using System.Xml.Linq;
using Observatory.Framework;
using Observatory.Framework.Interfaces;

namespace Observatory.Utils;

public static class ExportHandler
{
	private const string rels = "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/></Relationships>";

	private const string contentTypes = "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"xml\" ContentType=\"application/xml\"/><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Override PartName=\"/_rels/.rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/><Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/></Types>";

	private const string workbookRels = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet1.xml\"/></Relationships>";

	public static void ExportCSV(IObservatoryPlugin plugin)
	{
		//IL_017c: Unknown result type (might be due to invalid IL or missing references)
		//IL_0181: Unknown result type (might be due to invalid IL or missing references)
		//IL_0197: Unknown result type (might be due to invalid IL or missing references)
		//IL_0175: Unknown result type (might be due to invalid IL or missing references)
		//IL_003f: Unknown result type (might be due to invalid IL or missing references)
		//IL_004f: Unknown result type (might be due to invalid IL or missing references)
		//IL_0056: Expected Obj, but got Unknown
		//IL_01c2: Unknown result type (might be due to invalid IL or missing references)
		//IL_01c9: Unknown result type (might be due to invalid IL or missing references)
		//IL_01d5: Unknown result type (might be due to invalid IL or missing references)
		//IL_0237: Expected Obj, but got Unknown
		//IL_0238: Unknown result type (might be due to invalid IL or missing references)
		//IL_023e: Invalid comparison between Unknown and I4
		//IL_006d: Unknown result type (might be due to invalid IL or missing references)
		//IL_0074: Expected Obj, but got Unknown
		//IL_00c8: Unknown result type (might be due to invalid IL or missing references)
		//IL_00e2: Unknown result type (might be due to invalid IL or missing references)
		//IL_00e9: Expected Obj, but got Unknown
		string text = "\t";
		string filetype = "csv";
		byte[] array = plugin.ExportContent(text, ref filetype);
		if (array == null)
		{
			if (plugin.PluginUI.PluginUIType != PluginUI.UIType.Basic)
			{
				MessageBox.Show("Plugin " + plugin.Name + " does not use a basic data grid and does not provide an ExportContent method.", "Cannot Export", (MessageBoxButtons)0);
				return;
			}
			StringBuilder stringBuilder = new StringBuilder();
			ListView val = (ListView)((Control)(Panel)plugin.PluginUI.UI).Controls[0];
			foreach (ColumnHeader column in val.Columns)
			{
				ColumnHeader val2 = column;
				stringBuilder.Append(val2.Text + text);
			}
			stringBuilder.Append("\r\n");
			foreach (ListViewItem item in val.Items)
			{
				foreach (ListViewSubItem subItem in item.SubItems)
				{
					ListViewSubItem val3 = subItem;
					stringBuilder.Append(val3.Text + text);
				}
				stringBuilder.Append("\r\n");
			}
			array = Encoding.UTF8.GetBytes(stringBuilder.ToString());
		}
		SaveFileDialog val4 = new SaveFileDialog
		{
			Title = plugin.Name + " Export",
			Filter = ((filetype == "csv") ? "Tab-separated values (*.csv)|*.csv" : ("Plugin-specified file type (*." + filetype + ")|*." + filetype)),
			DefaultExt = filetype,
			InitialDirectory = Environment.GetFolderPath(Environment.SpecialFolder.Personal),
			FileName = $"Export-{plugin.ShortName}-{DateTime.Now:yyyy-MM-ddTHHmm}.{filetype}"
		};
		if ((int)((CommonDialog)val4).ShowDialog() == 1)
		{
			File.WriteAllBytes(((FileDialog)val4).FileName, array);
		}
	}

	public static void ExportXlsx(IObservatoryPlugin plugin)
	{
		//IL_0049: Unknown result type (might be due to invalid IL or missing references)
		//IL_0059: Unknown result type (might be due to invalid IL or missing references)
		//IL_005f: Expected Obj, but got Unknown
		//IL_00a2: Unknown result type (might be due to invalid IL or missing references)
		//IL_00a9: Expected Obj, but got Unknown
		//IL_0178: Unknown result type (might be due to invalid IL or missing references)
		//IL_02ad: Unknown result type (might be due to invalid IL or missing references)
		//IL_02b2: Unknown result type (might be due to invalid IL or missing references)
		//IL_02c8: Unknown result type (might be due to invalid IL or missing references)
		//IL_02d3: Unknown result type (might be due to invalid IL or missing references)
		//IL_02de: Unknown result type (might be due to invalid IL or missing references)
		//IL_02ea: Unknown result type (might be due to invalid IL or missing references)
		//IL_0345: Expected Obj, but got Unknown
		//IL_0347: Unknown result type (might be due to invalid IL or missing references)
		//IL_034d: Invalid comparison between Unknown and I4
		//IL_01c1: Unknown result type (might be due to invalid IL or missing references)
		//IL_01c8: Expected Obj, but got Unknown
		if (plugin.PluginUI.PluginUIType != PluginUI.UIType.Basic)
		{
			return;
		}
		XNamespace xNamespace = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
		XElement xElement = new XElement(xNamespace + "worksheet");
		XElement xElement2 = new XElement(xNamespace + "sheetData");
		ListView val = (ListView)((Control)(Panel)plugin.PluginUI.UI).Controls[0];
		XElement xElement3 = new XElement(xNamespace + "row", new XAttribute("r", "1"));
		char c = 'A';
		foreach (ColumnHeader column in val.Columns)
		{
			ColumnHeader val2 = column;
			xElement3.Add(new XElement(xNamespace + "c", new XAttribute("r", c + "1"), new XAttribute("t", "inlineStr"), new XElement(xNamespace + "is", new XElement(xNamespace + "t", val2.Text))));
			c = (char)(c + 1);
		}
		xElement2.Add(xElement3);
		int num = 2;
		foreach (ListViewItem item in val.Items)
		{
			c = 'A';
			XElement xElement4 = new XElement(xNamespace + "row", new XAttribute("r", num.ToString()));
			foreach (ListViewSubItem subItem in item.SubItems)
			{
				ListViewSubItem val4 = subItem;
				xElement4.Add(new XElement(xNamespace + "c", new XAttribute("r", c.ToString() + num), new XAttribute("t", "inlineStr"), new XElement(xNamespace + "is", new XElement(xNamespace + "t", val4.Text))));
				c = (char)(c + 1);
			}
			xElement2.Add(xElement4);
			num++;
		}
		xElement.Add(xElement2);
		SaveFileDialog val5 = new SaveFileDialog
		{
			Title = plugin.Name + " Export",
			Filter = "Office Open XML Spreadsheet (*.xlsx)|*.xlsx",
			DefaultExt = "xlsx",
			InitialDirectory = Environment.GetFolderPath(Environment.SpecialFolder.Personal),
			FileName = $"Export-{plugin.ShortName}-{DateTime.Now:yyyy-MM-ddTHHmm}.xlsx"
		};
		if ((int)((CommonDialog)val5).ShowDialog() == 1)
		{
			string text = Path.GetTempPath() + Path.DirectorySeparatorChar + Guid.NewGuid().ToString("D");
			Directory.CreateDirectory(text);
			Directory.CreateDirectory(text + Path.DirectorySeparatorChar + "_rels");
			Directory.CreateDirectory(text + Path.DirectorySeparatorChar + "xl");
			Directory.CreateDirectory(text + Path.DirectorySeparatorChar + "xl" + Path.DirectorySeparatorChar + "_rels");
			Directory.CreateDirectory(text + Path.DirectorySeparatorChar + "xl" + Path.DirectorySeparatorChar + "worksheets");
			File.WriteAllText(text + Path.DirectorySeparatorChar + "[Content_Types].xml", "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"xml\" ContentType=\"application/xml\"/><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Override PartName=\"/_rels/.rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/><Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/></Types>");
			File.WriteAllText(text + Path.DirectorySeparatorChar + "_rels" + Path.DirectorySeparatorChar + ".rels", "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/></Relationships>");
			File.WriteAllText(text + Path.DirectorySeparatorChar + "xl" + Path.DirectorySeparatorChar + "workbook.xml", workbook(plugin.ShortName + " Export"));
			File.WriteAllText(text + Path.DirectorySeparatorChar + "xl" + Path.DirectorySeparatorChar + "_rels" + Path.DirectorySeparatorChar + "workbook.xml.rels", "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet1.xml\"/></Relationships>");
			File.WriteAllText(text + Path.DirectorySeparatorChar + "xl" + Path.DirectorySeparatorChar + "worksheets" + Path.DirectorySeparatorChar + "sheet1.xml", xElement.ToString());
			ZipFile.CreateFromDirectory(text, ((FileDialog)val5).FileName);
			Directory.Delete(text, recursive: true);
		}
	}

	private static string workbook(string sheetName)
	{
		return "<workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><sheets><sheet name=\"" + sheetName + "\" sheetId=\"1\" r:id=\"rId1\"/></sheets></workbook>";
	}
}
