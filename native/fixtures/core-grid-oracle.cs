using System;using System.Collections;using System.Collections.Generic;using System.Linq;using System.Text.Json;using System.Windows.Forms;
namespace Observatory.UI { internal class PluginUIGrid:ListView {
private DefaultSorter _columnSorter=new();private Dictionary<Guid,List<ListViewItem>> _groupedItems=new();
public void SuspendDrawing(){}public void ResumeDrawing(){}
public void Add(string id,string text,string group){var row=new ListViewItem(text){Tag=id};Items.Add(row);if(group.Length!=0){var guid=Guid.Parse(group);if(!_groupedItems.ContainsKey(guid))_groupedItems[guid]=new();_groupedItems[guid].Add(row);}}
public void Click(int column)=>PluginListView_ColumnClick(null,new ColumnClickEventArgs(column));
public string[] Order()=>Items.Select(i=>i.Tag).ToArray();
	private void PluginListView_ColumnClick(object? sender, ColumnClickEventArgs e)
	{
		//IL_012a: Unknown result type (might be due to invalid IL or missing references)
		//IL_0134: Expected Obj, but got Unknown
		SuspendDrawing();
		if (e.Column == _columnSorter.SortColumn)
		{
			if (_columnSorter.Order == 1)
			{
				_columnSorter.Order = -1;
			}
			else
			{
				_columnSorter.Order = 1;
			}
		}
		else
		{
			_columnSorter.SortColumn = e.Column;
			_columnSorter.Order = 1;
		}
		if (_groupedItems.Count > 0)
		{
			ArrayList arrayList = new ArrayList();
			arrayList.AddRange((ICollection)((ListView)this).Items);
			foreach (KeyValuePair<Guid, List<ListViewItem>> groupedItem in _groupedItems)
			{
				foreach (ListViewItem item in groupedItem.Value.Skip(1))
				{
					arrayList.Remove(item);
				}
			}
			((ListView)this).Items.Clear();
			arrayList.Sort(_columnSorter);
			List<ListViewItem> list = new List<ListViewItem>();
			foreach (ListViewItem item2 in arrayList)
			{
				ListViewItem sortedItem = item2;
				list.Add(sortedItem);
				if (!_groupedItems.Where((KeyValuePair<Guid, List<ListViewItem>> g) => g.Value.First() == sortedItem).Any())
				{
					continue;
				}
				foreach (ListViewItem item3 in _groupedItems.Where((KeyValuePair<Guid, List<ListViewItem>> g) => g.Value.First() == sortedItem).First().Value.Skip(1).Reverse())
				{
					list.Add(item3);
				}
			}
			((ListView)this).Items.AddRange(list.ToArray());
		}
		else
		{
			((ListView)this).ListViewItemSorter = _columnSorter;
			((ListView)this).Sort();
			((ListView)this).ListViewItemSorter = null;
		}
		ResumeDrawing();
	}

}}
