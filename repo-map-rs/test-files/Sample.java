public class Sample {
    private String name;
    private int value;
    
    public Sample(String name, int value) {
        this.name = name;
        this.value = value;
    }
    
    public String getName() {
        return name;
    }
    
    public void setName(String name) {
        this.name = name;
    }
    
    public int getValue() {
        return value;
    }
    
    public void setValue(int value) {
        this.value = value;
    }
    
    public void processData() {
        System.out.println("Processing: " + name + " = " + value);
    }
    
    public static void main(String[] args) {
        Sample sample = new Sample("test", 42);
        sample.processData();
    }
}
