namespace System.Windows.Forms {
public class ListViewItem{public System.Collections.Generic.List<SubItem>SubItems=new();public string Tag;public ListViewItem(string text){SubItems.Add(new SubItem{Text=text});}}
public class SubItem{public string Text;}
public class ColumnClickEventArgs:System.EventArgs {public int Column;public ColumnClickEventArgs(int column){Column=column;}}
public class ItemsCollection:System.Collections.Generic.List<ListViewItem>{public void AddRange(ListViewItem[] items)=>base.AddRange(items);}
public class ListView {public ItemsCollection Items=new();public System.Collections.IComparer ListViewItemSorter;public void Sort(){Items.Sort((a,b)=>ListViewItemSorter.Compare(a,b));}}
}
namespace Observatory.Framework.Interfaces {public interface IObservatoryComparer {int SortColumn{get;set;}int Order{get;set;}}}
