using System;using System.Collections.Generic;
namespace System.Windows.Forms{
 public class Control{public List<Control>Controls=new();}public class Panel:Control{}
 public class ColumnHeader{public string Text;public ColumnHeader(string t){Text=t;}}
 public class ListViewItem{public List<ListViewSubItem>SubItems=new();public ListViewItem(params string[]s){foreach(var v in s)SubItems.Add(new(v));}public class ListViewSubItem{public string Text;public ListViewSubItem(string s){Text=s;}}}
 public class ListView:Control{public List<ColumnHeader>Columns=new();public List<ListViewItem>Items=new();}
 public enum MessageBoxButtons{OK}public static class MessageBox{public static void Show(string a,string b,MessageBoxButtons c){}}
 public class CommonDialog{public int ShowDialog()=>1;}public class FileDialog:CommonDialog{public string FileName{get=>SaveFileDialog.Target;set{}}public string Title,Filter,DefaultExt,InitialDirectory;}
 public class SaveFileDialog:FileDialog{public static string Target;}
}
namespace Observatory.Framework{public class PluginUI{public enum UIType{Basic,None}public UIType PluginUIType;public object UI;}}
namespace Observatory.Framework.Interfaces{public interface IObservatoryPlugin{string Name{get;}string ShortName{get;}global::Observatory.Framework.PluginUI PluginUI{get;}byte[]ExportContent(string delimiter,ref string type);}}
