using System;
using System.Collections;
using System.Linq;
using System.Windows.Forms;
using Observatory.Framework.Interfaces;

namespace Observatory.UI;

internal class DefaultSorter : IObservatoryComparer, IComparer
{
	private int ColumnToSort;

	private int OrderOfSort;

	private CaseInsensitiveComparer ObjectCompare;

	public int SortColumn
	{
		get
		{
			return ColumnToSort;
		}
		set
		{
			ColumnToSort = value;
		}
	}

	public int Order
	{
		get
		{
			return OrderOfSort;
		}
		set
		{
			OrderOfSort = value;
		}
	}

	public DefaultSorter()
	{
		ColumnToSort = 0;
		OrderOfSort = 0;
		ObjectCompare = new CaseInsensitiveComparer();
	}

	public int Compare(object? x, object? y)
	{
		//IL_0001: Unknown result type (might be due to invalid IL or missing references)
		//IL_0007: Expected Obj, but got Unknown
		//IL_0008: Unknown result type (might be due to invalid IL or missing references)
		//IL_000e: Expected Obj, but got Unknown
		ListViewItem val = (ListViewItem)x;
		ListViewItem val2 = (ListViewItem)y;
		if (OrderOfSort == 0)
		{
			return 0;
		}
		int num = NaturalCompare((val != null) ? val.SubItems[ColumnToSort].Text : null, (val2 != null) ? val2.SubItems[ColumnToSort].Text : null);
		if (OrderOfSort == 1)
		{
			return num;
		}
		return -num;
	}

	private static int NaturalCompare(string? x, string? y)
	{
		for (int i = 0; i <= x?.Length && i <= y?.Length; i++)
		{
			if (i == x.Length || i == y.Length)
			{
				if (x.Length <= y.Length)
				{
					if (y.Length <= x.Length)
					{
						return 0;
					}
					return -1;
				}
				return 1;
			}
			if (char.IsDigit(x[i]) && char.IsDigit(y[i]))
			{
				string text = x;
				int num = i;
				string text2 = new string(text.Substring(num, text.Length - num).TakeWhile((char c) => char.IsDigit(c)).ToArray());
				text = y;
				num = i;
				string text3 = new string(text.Substring(num, text.Length - num).TakeWhile((char c) => char.IsDigit(c)).ToArray());
				int num2 = Math.Max(text2.Length, text3.Length);
				string text4 = text2.PadLeft(num2, '0');
				string strB = text3.PadLeft(num2, '0');
				int num3 = text4.CompareTo(strB);
				if (num3 != 0)
				{
					return num3;
				}
				i += num2 - 1;
			}
			else if (x[i] != y[i])
			{
				if (x[i] <= y[i])
				{
					return -1;
				}
				return 1;
			}
		}
		return 0;
	}
}
