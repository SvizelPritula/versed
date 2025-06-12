namespace VisitorDemo;

public class DebugWriterVisitor<T>(T value, TextWriter textWriter, int indentLevel) : ITypeVisitor<T>
{
    public void VisitInt(Getter<T, int> get) => textWriter.WriteLine(get(ref value));
    public void VisitString(Getter<T, string> get) => textWriter.WriteLine(get(ref value));

    public void VisitStruct<S, M>(Getter<T, S> get, M type) where M : IStructType<S>
    {
        textWriter.WriteLine("{");
        type.Accept(new DebugWriterStructVisitor<S>(get(ref value), textWriter, indentLevel + 1));
        textWriter.Write(new string(' ', indentLevel * 2));
        textWriter.WriteLine("}");
    }
}

public class DebugWriterStructVisitor<T>(T value, TextWriter textWriter, int indentLevel) : IStructVisitor<T>
{
    public void VisitField<F, M>(string name, Getter<T, F> get, M visitor) where M : IType<F>
    {
        textWriter.Write(new string(' ', indentLevel * 2));
        textWriter.Write(name);
        textWriter.Write(": ");
        visitor.Accept(new DebugWriterVisitor<F>(get(ref value), textWriter, indentLevel));
    }
}
